import {useEffect} from 'react';
import {atom, DefaultValue, selector, useSetRecoilState} from 'recoil';
import {
    latestReadout,
    earliestReadTime,
    publisherList,
    Reading,
    readingRangesList,
} from '../state/sensors';
import RelayWS from '../WebSocket';

/// helper fn for splitting string by seperator once
function splitCmd(s: string) {
    const i = s.indexOf(' ');
    return [s.slice(0, i), s.slice(i + 1)];
}
interface Message {
    type: '/msg' | '/err' | string;
    text: string;
}

export const messageList = atom({
    key: 'MessageList_v1',
    default: [] as Message[],
});

export const messageArchive = atom({
    key: 'messageArchive_v1',
    default: [] as Message[],
});

export const newReadingHandler = selector<string>({
    key: 'readingListHandler_v1',
    set: ({set, get}, msg) => {
        if (msg instanceof DefaultValue) throw Error('not implemented');
        const [_, data] = splitCmd(msg);
        const reading = JSON.parse(data) as Reading;
        const cursor = `${reading.pub_id}|live`;
        if (get(earliestReadTime(reading.pub_id)) === null) {
            set(earliestReadTime(reading.pub_id), reading.read_time);
        }
        set(readingRangesList(cursor), (cur) =>
            cur === null ? [reading] : [...cur, reading]
        );
        set(latestReadout(reading.pub_id), reading);
    },
    get: () => {
        throw Error('use latestReadout atom directly');
    },
});

/// appends `Message` to messageList atom, DefaultValue moves list to msgArchive
export const messageListHandler = selector<string>({
    key: 'msgListHandler_v1',
    set: ({set, get}, msg) => {
        const m = get(messageList);
        if (msg instanceof DefaultValue) {
            set(messageArchive, (a) => [...a, ...m]);
            set(messageList, []);
            return;
        }
        const mData = splitCmd(msg);
        console.warn('message ', mData);
        const [type, text] = mData;
        const message = {type, text};
        set(messageList, (m) => [...m, message]);
    },
    get: () => {
        throw Error('use messageList atom directly');
    },
});

export const publisherListHandler = selector<string>({
    key: 'publisherListHandler_v1',
    set: ({set}, msg) => {
        if (msg instanceof DefaultValue) throw console.error(msg);
        const [_, arr] = splitCmd(msg);
        let data = JSON.parse(arr) as number[];
        set(publisherList, data);
    },
    get: () => {
        throw Error('use publishList atom directly');
    },
});

const useWebsocket = () => {
    const setPublisherList = useSetRecoilState(publisherListHandler);
    const setReadingList = useSetRecoilState(newReadingHandler);
    const setMsgList = useSetRecoilState(messageListHandler);
    useEffect(() => {
        if (RelayWS.ws != null) throw Error('Reinitialising Websocket!');
        RelayWS.connect();
        RelayWS.addListener('/list', setPublisherList);
        RelayWS.addListener('/reading', setReadingList);
        RelayWS.addListener('/msg', setMsgList);
        RelayWS.addListener('/err', setMsgList);
    }, [setPublisherList, setReadingList, setMsgList]);
};

export default useWebsocket;
