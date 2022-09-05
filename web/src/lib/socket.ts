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

async function onOpen(evt: Event) {
	console.log('ws opened', evt);
	void socket.emit('message', '<open>');

	// Step 1: get socket token by logging in via the auth server
	let res = await fetch('http://localhost:3000/login', {
		method: 'post',
		body: JSON.stringify({
			email: 'foo@example.com',
			password: '',
		}),
	}).then(r => r.json());

	// Step 2: send token to authenticate this socket
	sockette.send(JSON.stringify({ token: res.token }));

	setTimeout(() => {
		sockette.send(JSON.stringify({ 'Hi': `what's up?` }));
	}, 500);
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
}
