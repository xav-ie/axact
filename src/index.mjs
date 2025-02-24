// @ts-check
import { h, render } from "https://esm.sh/preact@10.26.2";
import htm from "https://esm.sh/htm@3.1.1";
const html = htm.bind(h);

const sleep = (/** @type {number} */ ms) =>
  new Promise((resolve) => setTimeout(resolve, ms));

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

const main = async () => {
  /** @type {number[]} */
  const cpuData = await fetch("/api/cpus").then((response) => response.json());

  const appElement = document.querySelector("div#app") || document.body;

  render(html`<${App} cpus=${cpuData} />`, appElement);
};

while (true) {
  await main().catch((error) => {
    console.error(error);
  });
  await sleep(1000);
}
