import {
    atom,
    atomFamily,
    DefaultValue,
    selector,
    useRecoilCallback,
} from 'recoil';
import {API_ADDRESS} from '../API_ADDRESS';

export const latestReadout = atomFamily<Reading | null, number>({
    key: 'latestReadout_v1',
    default: null,
});

export const publisherList = atom({
    key: 'publisherList_v1',
    default: [] as number[],
});

/// cache the chronologically earliest read_time loaded for a sensor
/// indexed by the sensor's publisher id (pub_id)
/// initially set by sensor's first /reading from websocket
/// used by getEarlierReadings to get time of earliest reading for creating new cursors
export const earliestReadTime = atomFamily<number | null, number>({
    key: 'lastReadTime',
    default: null,
});

/// contains a set of cursors that index the readingRangesList
/// indexed by sensor id from publisherList
export const readingCursorSet = atomFamily<Set<string>, number>({
    key: 'readingCursorList_v1',
    default: (pubId: number) => new Set([`${pubId}|live`]),
});

export interface Reading {
    pub_id: number;
    eco2: number;
    evtoc: number;
    read_time: number;
    start_time: number;
    increment: string;
}

/// contains ranges of readings for different sensors, set by getEarlierReadings
/// indexed by `{sensor_id}|{before}|{limit}` from `readingCursorList`
export const readingRangesList = atomFamily<null | Reading[], string>({
    key: 'readingRangesList_v1',
    default: null,
});

/// requests reading based on cursor parameters
/// cursor is formatted as `{sensor_id}|{before}|{limit}`
export function useSensorReadingsAPI() {
    return useRecoilCallback(
        ({snapshot, set}) =>
            async ({pubId, limit}: {pubId: number; limit: number}) => {
                if (!API_ADDRESS) throw Error('no API ADDRESS');
                const readTime = await snapshot.getPromise(
                    earliestReadTime(pubId)
                );
                if (readTime === null) throw Error('no readings for ' + pubId);
                const cursor = `${pubId}|${readTime}|${limit}`;
                const existing = await snapshot.getPromise(
                    readingRangesList(cursor)
                );
                if (existing !== null) return;
                const query = new URLSearchParams({
                    pub_id: String(pubId),
                    before: String(readTime),
                    limit: String(limit),
                });
                fetch(`${API_ADDRESS}sensors/readings?${query}`, {
                    method: 'GET',
                    headers: {
                        Accept: 'application/json',
                    },
                }).then((res) => {
                    if (!res.ok) throw res;
                    res.json().then((data: Reading[]) => {
                        if (Array.isArray(data)) {
                            if (data.length) {
                                const fReading = data[0];
                                set(
                                    earliestReadTime(pubId),
                                    fReading.read_time
                                );
                            }
                            set(readingCursorSet(pubId), (s) => {
                                if (s.has(cursor)) return s;
                                s = new Set(s);
                                s.add(cursor);
                                return s;
                            });
                            set(readingRangesList(cursor), data);
                        }
                    });
                });
            }
    );
}
