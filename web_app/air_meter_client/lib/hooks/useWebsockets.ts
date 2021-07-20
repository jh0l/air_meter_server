import {useEffect} from 'react';
import {
    atom,
    atomFamily,
    DefaultValue,
    selector,
    useSetRecoilState,
} from 'recoil';
import RelayWS from '../WebSocket';

/// helper fn for splitting string by seperator once
function splitCmd(s: string) {
    const i = s.indexOf(' ');
    return [s.slice(0, i), s.slice(i + 1)];
}

export const subscribedData = atomFamily<string | null, number>({
    key: 'subscribedData_v1',
    default: null,
});

export const publisherList = atom({
    key: 'publisherList_v1',
    default: [] as number[],
});

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

export const subscribedDataHandler = selector<string>({
    key: 'readingListHandler_v1',
    set: ({set}, msg) => {
        if (msg instanceof DefaultValue) throw Error('not implemented');
        const [_, data] = splitCmd(msg);
        const reading = JSON.parse(data);
        set(subscribedData(reading.pub_id), msg);
    },
    get: () => {
        throw Error('use subscribedData atom directly');
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
    const setReadingList = useSetRecoilState(subscribedDataHandler);
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
