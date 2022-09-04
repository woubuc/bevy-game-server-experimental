const plugin = require('tailwindcss/plugin');

/** @type {import('tailwindcss').Config} */
module.exports = {
	content: [
		'src/app.html',
		'src/**/*.{ts,svelte}',
	],
	theme: {
		extend: {
			fontFamily: {
				body: `'Inter', sans-serif`,
				mono: `'Courier Prime Code', monospace`,
			},
		},
	},
	plugins: [
		plugin(function ({ addVariant }) {
			addVariant('hocus', ['&:hover', '&:focus', '&:active']);
			addVariant('group-hocus', ['.group:hover &', '.group:focus &', '.group:active &']);
		}),
	],
};
