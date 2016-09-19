"use strict";
const String_1 = require("./String");
var fs = require('fs');
class Frump {
    constructor() {
        this.Title = "";
        this.Description = "";
        this.Team = [];
        this.Tasks = [];
    }
}
exports.Frump = Frump;
class FrumpParser {
    constructor() {
        this.current = {
            section: "",
            id: 0
        };
    }
    Parse(file) {
        this.frump = new Frump();
        this.readFrump(file);
        this.lines.forEach(line => {
            if (/^\#[^\#].*/.test(line)) {
                this.current.section = "header";
                this.frump.Title = this.parseHeader(line);
            }
            else if (/^\#\#[^\#].*Tasks/.test(line)) {
                this.current.section = "tasks";
            }
            else if (/^\#\#[^\#].*Team/.test(line)) {
                this.current.section = "team";
            }
            else if (/^\#\#\#[^\#].*/.test(line)) {
                this.frump.Tasks.push(this.parseTask(line));
            }
            else if (/^\*/.test(line)) {
                if (this.current.section == "team") {
                    this.frump.Team.push(this.parseMember(line));
                }
            }
            else if (/^Comments.*/.test(line)) {
                this.current.section = "tasks.comments";
            }
            else if (/^>.*/.test(line)) {
                if (this.current.section == "tasks.comments") {
                    this.frump.Tasks[this.current.id].Comments.push(this.parseComment(line));
                }
            }
            else if (/^_.*/.test(line)) {
                if (this.current.section == "tasks") {
                    this.frump.Tasks[this.current.id].Params = this.parseTaskParams(line);
                }
            }
            else {
                if (this.current.section == "tasks") {
                    this.frump.Tasks[this.current.id].Descriiption += line;
                }
                if (this.current.section == "header") {
                    this.frump.Description += line;
                }
            }
        });
        return this.frump;
    }
    parseHeader(line) {
        return String_1.String.Repalce(line, "#", "");
    }
    parseMember(line) {
        return {
            Name: String_1.String.SubstringBetween(line, "*", "<"),
            Email: String_1.String.SubstringBetween(line, "<", ">"),
            Role: String_1.String.Trim(line.slice(line.indexOf(">") + 1).replace(/^\s{0,}\-/, ""))
        };
    }
    parseTask(line) {
        var main = String_1.String.Trim(String_1.String.SubstringBetween(line, "###", "-"));
        var task = {
            Subject: String_1.String.Trim(line.slice(line.indexOf("-")).replace(/^\-\s{0,}/, "")),
            Id: +main.split(" ")[1],
            Comments: [],
            Descriiption: "",
            Params: [],
            Type: main.split(" ")[0]
        };
        return task;
    }
    parseTaskParams(line) {
        var props = [];
        line = line.slice(1, -1);
        var propsArray = line.split(",");
        propsArray.forEach(propString => {
            var prop = propString.split(":");
            props.push({
                Name: String_1.String.Trim(prop[0]),
                Value: String_1.String.Trim(prop[1])
            });
        });
        return props;
    }
    parseComment(line) {
        return {
            By: String_1.String.Trim(line.slice(line.indexOf(">") + 1, line.indexOf(":"))),
            Message: String_1.String.Trim(line.slice(line.indexOf(":") + 1))
        };
    }
    parseText(line) {
        return String_1.String.NormalizeSpace(line);
    }
    readFrump(path) {
        var fileText = fs.readFileSync(path).toString();
        var lines = fileText.split(/\r?\n/);
        this.lines = [];
        lines.forEach(line => {
            line = String_1.String.NormalizeSpace(String_1.String.Trim(line));
            if (line.length > 0) {
                this.lines.push(line);
            }
        });
    }
}
exports.FrumpParser = FrumpParser;
//# sourceMappingURL=Frump.js.map