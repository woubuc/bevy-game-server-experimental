import Emittery from 'emittery';
import Sockette from 'sockette';

export const socket = new Emittery<{ message: string }>();

const sockette = new Sockette('ws://localhost:3333', {
	onopen: onOpen,
	onclose: onClose,
	onerror: onError,
	onreconnect: onReconnect,
	onmessage: onMessage,
});

function onOpen(evt: Event) {
	console.log('ws opened', evt);
	void socket.emit('message', '<open>');
	sockette.send(JSON.stringify({ 'Hi': `what's up?` }));
}

function onClose(evt: CloseEvent) {
	console.warn('ws closed', evt.reason);
	void socket.emit('message', '<close>');
}

function onReconnect(evt: Event) {
	console.info('ws reconnecting', evt);
	void socket.emit('message', '<reconnect>');
}

function onError(evt: Event) {
	console.error('ws error', evt);
	void socket.emit('message', '<error>');
}

function onMessage(evt: MessageEvent) {
	void socket.emit('message', evt.data);
	console.log('ws message', evt.data);
}
