<script lang="ts">
import format from 'date-fns/format';
import { onDestroy, onMount } from 'svelte';
import { socket } from '../lib/socket';

let messages: string[] = [];
onMount(() => {
	messages = [];
	addMessage('<connect>');
	socket.on('message', addMessage);
});
onDestroy(() => {
	messages = [];
	socket.off('message', addMessage);
});

function addMessage(msg: string) {
	let d = new Date();
	messages.unshift(`[${ format(d, 'HH:ii:ss') }] ${ msg }`);
	messages = messages;
}
</script>

<div class="px-12 py-10 divide-y divide-gray-300">
	<h1 class="border-b-2 border-gray-300 font-bold text-lg">Messages</h1>
	{#each messages as message}
		<pre class="py-1.5 font-mono text-xs">{message}</pre>
	{/each}
</div>
