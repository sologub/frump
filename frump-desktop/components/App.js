"use strict";
const React = require("react");
const Frump_1 = require("../modules/Frump");
const AppHeader_1 = require("./AppHeader");
const AppContent_1 = require("./AppContent");
class App extends React.Component {
    constructor(props) {
        super(props);
        var parser = new Frump_1.FrumpParser();
        this.state = {
            frump: parser.Parse("C:/temp/frump.md")
        };
    }
    taskDetails(id) {
        var task = this.state.frump.Tasks.find(x => x.Id == id);
    }
    refresh() {
    }
    render() {
        return React.createElement("div", {className: "container-fluid"}, React.createElement(AppHeader_1.AppHeader, null), React.createElement(AppContent_1.AppContent, {frump: this.state.frump}));
    }
}
exports.App = App;
//# sourceMappingURL=App.js.map