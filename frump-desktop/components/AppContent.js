"use strict";
const React = require("react");
class AppContent extends React.Component {
    render() {
        console.log(this.props.frump);
        return React.createElement("div", {className: "row"}, React.createElement("div", {className: "col s3 m2"}, React.createElement("h3", {className: "page-header"}, "Team"), React.createElement("div", {className: "collection"}, this.props.frump.Team.map(function (member) {
            return React.createElement("a", {key: member.Email, href: "#", className: "collection-item"}, member.Name, React.createElement("span", {className: "badge"}, member.Role));
        }))), React.createElement("div", {className: "col s9 m10"}, React.createElement("h3", {className: "page-header"}, "Tasks: ", this.props.frump.Title), React.createElement("div", {className: "row"}, React.createElement("div", {className: "col s12"}, this.props.frump.Tasks.map(function (task) {
            return React.createElement("div", {key: task.Id, className: task.Type == "Bug" || task.Type == "bug" ? "card red darken-3 hoverable" : "card blue-grey darken-1 hoverable"}, React.createElement("div", {className: "card-content white-text"}, React.createElement("span", {className: "card-title"}, task.Subject, " ", React.createElement("span", {className: "right amber-text text-darken-4"}, task.Type)), React.createElement("p", null, task.Descriiption)), React.createElement("div", {className: "card-action"}, task.Params.map(function (param) {
                return React.createElement("a", {key: param.Name}, param.Name, " - ", param.Value);
            })));
        })))));
    }
}
exports.AppContent = AppContent;
//# sourceMappingURL=AppContent.js.map