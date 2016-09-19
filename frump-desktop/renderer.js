"use strict";
const React = require("react");
const ReactDom = require("react-dom");
const react_router_1 = require("react-router");
const App_1 = require("./components/App");
class Main extends React.Component {
    render() {
        return React.createElement(react_router_1.Router, {history: withExampleBasename(react_router_1.browserHistory, __dirname)}, React.createElement(react_router_1.Route, {path: "/", component: App_1.App}, React.createElement(react_router_1.Route, {path: "category/:category", components: { content: Category, sidebar: CategorySidebar }}, React.createElement(react_router_1.Route, {path: ":item", component: Item}))));
    }
}
exports.Main = Main;
ReactDom.render(React.createElement(App_1.App, null), document.querySelector("main"));
//# sourceMappingURL=renderer.js.map