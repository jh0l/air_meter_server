export const WS_ADDRESS =
    process.env.WS_ADDRESS ||
    (() => {
        throw new Error('websocket address not supplied');
    })();
export const API_ADDRESS = process.env.API_ADDRESS;
