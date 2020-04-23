/* globals gauge*/
"use strict";
const { openBrowser, write, closeBrowser, goto, screenshot, text, below, textBox, button, click, clear, $, toLeftOf, goBack } = require('taiko');
const assert = require("assert");
const headless = process.env.headless_chrome.toLowerCase() === 'true';

beforeSuite(async () => {
    await openBrowser({ headless: headless })
});

afterSuite(async () => {
    await closeBrowser();
});

gauge.customScreenshotWriter = async function () {
    const screenshotFilePath = path.join(process.env['gauge_screenshots_dir'], `screenshot-${process.hrtime.bigint()}.png`);
    await screenshot({ path: screenshotFilePath });
    return path.basename(screenshotFilePath);
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

step("Login with testing creds", async function () {
    await goto('http://127.0.0.1:8080/login');
    await click($("#username"))
    await write("matthew")
    await click($("#password"))
    await write("thisisfortesting")
    await click(button("login"))
});

step("Click the edit button next to pizza", async function () {
    await click(button(toLeftOf("pizza")))
});

step("Press the browser's back button", async function () {
    await goBack();
});