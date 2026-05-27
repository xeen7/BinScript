const s = '{"name": "test", "value": 42, "active": true, "missing": null}';
const tape = JSON.parse(s);

const v1 = tape.value;
const v2 = tape.active;
const v3 = tape.missing;

if (v1) {
    console.log(v1);
}
if (v2) {
    console.log(99);
}
console.log(88);
