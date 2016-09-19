"use strict";
const React = require("react");
class TaskDetails extends React.Component {
    render() {
        return React.createElement("div", {className: "row"}, React.createElement("div", {className: "col s12"}, React.createElement("h3", {className: "page-header"}, this.props.Subject), React.createElement("p", null, this.props.Descriiption), React.createElement("div", {className: "divider"}), React.createElement("div", {className: "row"}, React.createElement("div", {className: "col s12"}, React.createElement("h4", {className: "page-header"}, "Properties"), this.props.Params.map(function (param, i) {
            return React.createElement("div", {key: i, class: "chip"}, param.Name, ": ", param.Value, React.createElement("i", {class: "close material-icons"}, "close"));
        }))), React.createElement("div", {className: "row"}, React.createElement("div", {className: "col s12"}, React.createElement("h4", {className: "page-header"}, "Comments"), this.props.Comments.map(function (comment, i) {
            return React.createElement("div", {key: i, className: "card-panel teal"}, React.createElement("span", {className: "white-text"}, React.createElement("b", null, comment.By), comment.Message));
        })))));
    }
}
exports.TaskDetails = TaskDetails;
//# sourceMappingURL=TaskDetails.js.map