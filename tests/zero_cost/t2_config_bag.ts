function formatString(text: string, options: any) {
    if (options.uppercase) {
        text = text.toUpperCase();
    }
    if (options.trim) {
        text = text.trim();
    }
    return text;
}

function render() {
    let title = formatString("  hello world  ", { uppercase: true, trim: true });
    console.log("Rendered title: " + title);
}

render();
