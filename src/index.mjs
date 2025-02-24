// @ts-check
import { h, render } from "https://esm.sh/preact@10.26.2";
import htm from "https://esm.sh/htm@3.1.1";
const html = htm.bind(h);

const formatNumber = (/** @type {number} */ num) =>
  String(Math.round(num)).padStart(2, "0");

/**
 * @param {{ value: number; }} props
 */
const ProgressBar = ({ value }) => {
  return html`
    <div role="progressbar" aria-valuenow="${formatNumber(value)}">
      ${value.toFixed(2).padStart(5, " ")}
    </div>
  `;
};

/**
 * @param {{ cpus: number[]; }} props
 */
const App = ({ cpus }) => {
  return html`
    <ol>
      ${cpus.map((cpu) => {
        return html`<li><${ProgressBar} value=${cpu} /></li>`;
      })}
    </ol>
  `;
};

const main = (/** @type {number[]} */ cpuData) => {
  const appElement = document.querySelector("div#app") || document.body;

  render(html`<${App} cpus=${cpuData} />`, appElement);
};

const url = new URL("/api/realtime_cpus", location.href);
url.protocol = url.protocol.replace("http", "ws");
const ws = new WebSocket(url.href);
ws.addEventListener("message", (event) => {
  main(JSON.parse(event.data));
});
