/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,js,svelte,ts}", // make sure this is correct for your project
  ],
  safelist: [
    "text-black",
    "text-red-500",
  ],
}