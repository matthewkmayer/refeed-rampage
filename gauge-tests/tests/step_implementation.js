/* globals gauge*/
"use strict";
const { openBrowser,write, closeBrowser, goto, press, screenshot, text, focus, textBox, toRightOf, click } = require('taiko');
const assert = require("assert");
const headless = process.env.headless_chrome.toLowerCase() === 'true';

beforeSuite(async () => {
    await openBrowser({ headless: headless })
});

afterSuite(async () => {
    await closeBrowser();
});

gauge.screenshotFn = async function() {
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


