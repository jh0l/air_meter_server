const isProd = process.env.NODE_ENV === 'production';
module.exports = {
    reactStrictMode: true,
    assetPrefix: isProd ? '/static' : '',
    env: {
        WS_ADDRESS: isProd ? '/' : 'ws://127.0.0.1:8080/ws/',
        API_ADDRESS: isProd ? '/api/' : 'http://127.0.0.1:8080/api/',
    },
};
