"use strict";
const React = require("react");
class AppHeader extends React.Component {
    render() {
        return React.createElement("nav", null, React.createElement("div", {className: "nav-wrapper blue-grey darken-4"}, React.createElement("a", {href: "/", className: "brand-logo left"}, " Frump"), React.createElement("ul", {id: "nav-mobile", className: "right "}, React.createElement("li", null, React.createElement("a", {href: "#"}, "Settings")))));
    }
}
exports.AppHeader = AppHeader;
//# sourceMappingURL=AppHeader.js.map