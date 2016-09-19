"use strict";
const S = require("string");
class String {
    static Trim(value) {
        return value.trim();
    }
    static NormalizeSpace(value) {
        return value;
    }
    static Substring(value, start, end) {
        return value.substring(start, end);
    }
    static SubstringBetween(value, left, right) {
        return S(value).between(left, right).s;
    }
    static Repalce(value, oldValue, newValue) {
        return S(value).replaceAll(oldValue, newValue).s;
    }
    static IndexOf(value, search) {
        return value.indexOf(search);
    }
    static Split(value, separator) {
        return value.split(separator);
    }
    static Contains(value, search) {
        return value.indexOf(search) >= 0;
    }
}
exports.String = String;
//# sourceMappingURL=String.js.map