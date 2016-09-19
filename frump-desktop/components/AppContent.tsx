import React = require("react");
import {Frump} from "../modules/Frump"

interface AppContentProps{
    frump: Frump
}

export class AppContent extends React.Component<AppContentProps, {}>{
    render() {
        console.log(this.props.frump);
        return <div className="row">
            <div className="col s3 m2">
                <h3 className="page-header">Team</h3>
                <div className="collection">
                    {
                        this.props.frump.Team.map(function (member) {
                            return <a key={member.Email} href="#" className="collection-item">{member.Name}<span className="badge">{member.Role}</span></a>
                        })
                    }
                </div>
            </div>
            <div className="col s9 m10">
                <h3 className="page-header">Tasks: {this.props.frump.Title}</h3>
                <div className="row">
                    <div className="col s12">
                        {this.props.frump.Tasks.map(function (task) {
                            return <div key={task.Id} className={task.Type == "Bug" || task.Type == "bug" ? "card red darken-3 hoverable" : "card blue-grey darken-1 hoverable"}>
                                <div className="card-content white-text">
                                    <span className="card-title">{task.Subject} <span className="right amber-text text-darken-4">{task.Type}</span></span>
                                    <p>{task.Descriiption}</p>
                                </div>
                                <div className="card-action">
                                    {
                                        task.Params.map(function (param) {
                                            return <a key={param.Name}>{param.Name} - {param.Value}</a>
                                         })
                                    }
                                </div>
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>;
    }
}