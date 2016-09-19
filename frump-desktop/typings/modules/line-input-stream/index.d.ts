// Generated by typings
// Source: https://raw.githubusercontent.com/types/npm-line-input-stream/cf8f4822ed36c780d83f88dfe30668f79201a3c7/src/index.d.ts
declare module 'line-input-stream' {
import * as events from 'events';
import * as net from 'net';

interface LineInputStream extends events.EventEmitter {
	addListener(type: string, listener: () => void);
	removeListener(type: string, listener: () => void);
	removeAllListeners(type: string);
	pause();
	resume();
	destroy();
	setEncoding(encoding: string);
	setDelimiter(delimiter: string);
	// get readable(): boolean;
	// get
}

function constructor(socket: net.Socket): LineInputStream;

export = constructor;
}