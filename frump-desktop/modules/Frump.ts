import {String} from "./String";
var fs = require('fs');

export interface IMember {
    Name: string,
    Email: string,
    Role: string
}

export interface IComment {
    By: string,
    Message: string
}

export interface IProp {
    Name: string,
    Value: string
}

export interface ITask {
    Id: number,
    Type: string,
    Subject: string,
    Descriiption: string,
    Params: Array<IProp>,
    Comments: Array<IComment>
}

export interface IFrump {
    Title: string,
    Description: string,
    Team: Array<IMember>
}

export class Frump {
    Title: string = "";
    Description: string = "";
    Team: Array<IMember> = [];
    Tasks: Array<ITask> = [];
}

export class FrumpParser {
    current: {
        section: string,
        id: number
    }
    frump: Frump;

    constructor() {
        this.current = {
            section: "",
            id: 0
        }
    }

    private lines: Array<string>;

    public Parse(file: string): Frump {
        this.frump = new Frump();
        this.readFrump(file);
        this.lines.forEach(line => {
            if (/^\#[^\#].*/.test(line)) {
                this.current.section = "header";
                this.frump.Title = this.parseHeader(line);
            }
            else if (/^\#\#[^\#].*Tasks/.test(line)) {
                this.current.section = "tasks"
            }
            else if (/^\#\#[^\#].*Team/.test(line)) {
                this.current.section = "team"
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
    
    // -- Private ----------------------------------------------------------
    private parseHeader(line: string): string {
        return String.Repalce(line, "#", "");
    }

    private parseMember(line: string): IMember {
        return {
            Name: String.SubstringBetween(line, "*", "<"),
            Email: String.SubstringBetween(line, "<", ">"),
            Role: String.Trim(line.slice(line.indexOf(">") + 1).replace(/^\s{0,}\-/, ""))
        };
    }

    private parseTask(line: string): ITask {
        var main: string = String.Trim(String.SubstringBetween(line, "###", "-"));
        var task = {
            Subject: String.Trim(line.slice(line.indexOf("-")).replace(/^\-\s{0,}/, "")),
            Id: +main.split(" ")[1],
            Comments: [],
            Descriiption: "",
            Params: [],
            Type: main.split(" ")[0]
        };
        return task;
    }

    private parseTaskParams(line: string): Array<IProp> {
        var props: Array<IProp> = [];
        line = line.slice(1, -1);
        var propsArray = line.split(",");
        propsArray.forEach(propString => {
            var prop = propString.split(":");
            props.push({
                Name: String.Trim(prop[0]),
                Value: String.Trim(prop[1])
            });
        });
        return props;
    }

    private parseComment(line: string): IComment {
        return {
            By: String.Trim(line.slice(line.indexOf(">") + 1, line.indexOf(":"))),
            Message: String.Trim(line.slice(line.indexOf(":") + 1))
        };
    }

    private parseText(line: string): string {
        return String.NormalizeSpace(line);
    }

    // -- End - Private ----------------------------------------------------

    private readFrump(path: string) {
        var fileText = fs.readFileSync(path).toString();
        var lines: Array<string> = fileText.split(/\r?\n/);
        this.lines = [];
        lines.forEach(line => {
            line = String.NormalizeSpace(String.Trim(line));
            if (line.length > 0) {
                this.lines.push(line);
            }
        });
    }
}


// //-- helpers --------------------------------------------------------------------

// var normalizeSpace = function (input) {
//     while (input.indexOf("  ") >= 0) {
//         input = S(input).replaceAll("  ", " ").s;
//     }
//     return input;
// }
// var trim = function (input) {
//     return S(input).trim().s;
// }
// var replaceAll = function (input, oldValue, newValue) {
//     return S(input).replaceAll(oldValue, newValue).s;
// }
// //-- end helpers ----------------------------------------------------------------

// var FrumpParser = {
//     Parse: {
//         FileHead: function (line) {
//             return trim(replaceAll(line, "#", ""));
//         },
//         TeamMember: function (member) {
//             return {
//                 Name: normalizeSpace(S(S(member).between("*", "<").s).trim().s),
//                 Role: normalizeSpace(S(S(member).left(member.indexOf('>') - member.length).s).replaceAll("> -", "").s),
//                 Email: normalizeSpace(S(S(member).between("<", ">").s).trim().s)
//             }
//         },
//         TaskHead: function (title) {
//             var head = normalizeSpace(S(title).between("### ", " -").s);
//             var type = head.split(" ")[0];
//             var id = head.split(" ")[1];
//             return {
//                 Id: id,
//                 Type: type,
//                 Subject: normalizeSpace(S(S(title).replaceAll("### " + type + " " + id + " -", "").s).trim().s),
//                 Description: ""
//             };
//         },
//         TaskParams: function (line) {
//             line = S(S(line).left(line.length - 1).s).right(line.length - 2).s;
//             var paramsArray = line.split(',');
//             var params = [];
//             for (var i = 0; i < paramsArray.length; i++) {
//                 var param = {
//                     Name: trim(paramsArray[i].split(':')[0]),
//                     Value: trim(paramsArray[i].split(':')[1])
//                 };
//                 params.push(param);
//             }
//             return params;
//         },
//         TaskComments: function (line) {
//             line = S(line).replaceAll("> ", "").s;
//             return {
//                 By: S(S(line).left(line.indexOf(":")).s).trim().s,
//                 Message: S(S(line).right(line.length - line.indexOf(":") - 1).s).trim().s,
//             }
//         },
//         FreeText: function (line) {
//             return S(normalizeSpace(line)).trim().s;

//         }
//     }
// }

// var Sections = {
//     Current: "",
//     CurrentID: ""
// }

// export class Reader {
//     static Read(callback: Function) {
//         var lines = [];
//         var lineReader = require('readline').createInterface({
//             input: require('fs').createReadStream(filePath)
//         });
//         lineReader.on('line', function (line) {
//             line = S(line).trim().s;
//             lines.push(line);
//         }).on('close', () => {
//             var current = "Title";
//             for (var i = 0; i < lines.length; i++) {
//                 var line = S(normalizeSpace(lines[i])).trim().s;

//                 // Project description
//                 if (S(line).startsWith("# ")) {
//                     Frump.Title = FrumpParser.Parse.FileHead(line);
//                     Sections.Current = "Title";
//                 }

//                 // Section, one of Team or Tasks
//                 else if (S(line).startsWith("## ")) {
//                     //
//                 }

//                 // task subject
//                 else if (S(line).startsWith("### ")) {
//                     Sections.Current = "Task";
//                     var task = FrumpParser.Parse.TaskHead(line);
//                     Sections.CurrentID = task.Type + "-" + task.Id;
//                     Frump.Tasks[Sections.CurrentID] = task;
//                 }

//                 // team member
//                 else if (S(line).startsWith("*")) {
//                     var member = FrumpParser.Parse.TeamMember(line);
//                     Frump.Team.push(member);
//                 }

//                 // task properties
//                 else if (S(line).startsWith("_")) {
//                     Sections.Current = "TaskParams";
//                     var params = FrumpParser.Parse.TaskParams(line);
//                     Frump.Tasks[Sections.CurrentID].Params = params;
//                 }

//                 // task comments
//                 else if (S(line).startsWith(">")) {
//                     Sections.Current = "TaskComments";
//                     var comment = FrumpParser.Parse.TaskComments(line);
//                     if (!Frump.Tasks[Sections.CurrentID].Commets) Frump.Tasks[Sections.CurrentID].Commets = [];
//                     Frump.Tasks[Sections.CurrentID].Commets.push(comment);
//                 }

//                 // Free text
//                 else {
//                     switch (Sections.Current) {
//                         case "Title": Frump.Description += line; break;
//                         case "Task":
//                             if (line && line.length > 0) {
//                                 Frump.Tasks[Sections.CurrentID].Description += line + "\n";
//                             }
//                             break;
//                     }
//                 }
//             }
//             var frump = JSON.stringify(Frump);
//             callback(null, frump);
//         });
//     }
// }

