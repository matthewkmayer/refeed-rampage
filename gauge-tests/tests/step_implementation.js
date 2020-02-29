/* globals gauge*/
"use strict";
const { openBrowser, write, closeBrowser, goto, screenshot, text, below, textBox, click, clear } = require('taiko');
const assert = require("assert");
const headless = process.env.headless_chrome.toLowerCase() === 'true';

beforeSuite(async () => {
    await openBrowser({ headless: headless })
});

afterSuite(async () => {
    await closeBrowser();
});

gauge.screenshotFn = async function () {
    return await screenshot({ encoding: 'base64' });
};


step("Page contains <content>", async (content) => {
    assert.ok(await text(content).exists());
});

step("Goto refeed rampage home", async () => {
    await goto('http://127.0.0.1:8080');
});
step("Click <clicker>", async (clicker) => {
    await click(clicker)
});

step("Fill out wings as a new meal", async function () {
    await (click(textBox(below("meal name"))))
    await (write("wings"))
    await (click(textBox(below("meal description"))))
    await (write("delicious wings"))
    await (click("make it"))
});

step("Update the name to <n> and description to <d>", async function (n, d) {
    await (click(textBox(below("meal name"))))
    await (clear(textBox(below("meal name"))))
    await (write(n))
    await (click(textBox(below("meal description"))))
    await (clear(textBox(below("meal description"))))
    await (write(d))
});