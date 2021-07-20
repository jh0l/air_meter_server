import {SetterOrUpdater} from 'recoil';
import {WS_ADDRESS} from './API_ADDRESS';

export default class RelayWS {
    static commands = ['/list', '/join', '/msg', '/err', '/reading'];
    static ws: WebSocket | null = null;
    static listeners: Map<string, SetterOrUpdater<unknown>> = new Map();

    static connect() {
        let ws = new WebSocket(WS_ADDRESS);
        ws.onopen = () => {
            console.log('WS connected');
            ws?.send('/list');
        };
        RelayWS.ws = ws;
        ws.onmessage = ({data}: {data: string}) => {
            const [command] = data.split(' ');
            const handler = RelayWS.listeners.get(command);
            if (handler) handler(data);
            else console.log('unhandled ws message: ', data);
        };
    }

    static addListener(command: string, listener: SetterOrUpdater<any>) {
        if (!RelayWS.commands.includes(command))
            throw Error(command + ' not a command');
        RelayWS.listeners.set(command, listener);
    }

    static sendList() {
        RelayWS.ws?.send('/list');
    }

    static sendJoin(pubId: number) {
        const data = {pub_id: pubId};
        RelayWS.ws?.send(`/join ${JSON.stringify(data)}`);
    }
}
