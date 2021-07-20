import '../styles/globals.css';
import 'tailwindcss/tailwind.css';
import type {AppProps} from 'next/app';
import useWebsocket from '../lib/hooks/useWebsockets';
import {RecoilRoot} from 'recoil';

function UseWebsockets() {
    useWebsocket();
    return null;
}

function MyApp({Component, pageProps}: AppProps) {
    return (
        <RecoilRoot>
            <UseWebsockets />
            <Component {...pageProps} />;
        </RecoilRoot>
    );
}
export default MyApp;
