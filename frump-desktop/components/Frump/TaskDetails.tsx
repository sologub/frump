import React = require("react");
import {ITask} from "../../modules/frump";

export class TaskDetails extends React.Component<ITask, {}>{
    render() {
        return <div className="row">
            <div className="col s12">
                <h3 className="page-header">{this.props.Subject}</h3>
                <p>{this.props.Descriiption}</p>
                <div className="divider"></div>
                <div className="row">
                    <div className="col s12">
                        <h4 className="page-header">Properties</h4>
                        {this.props.Params.map(function (param, i) {
                            return <div key={i} class="chip">
                                {param.Name}: {param.Value}
                                <i class="close material-icons">close</i>
                            </div>
                        })}
                    </div>
                </div>
                <div className="row">
                    <div className="col s12">
                        <h4 className="page-header">Comments</h4>
                        {this.props.Comments.map(function (comment, i) {
                            return <div key={i} className="card-panel teal">
                                <span className="white-text"><b>{comment.By}</b>{comment.Message}</span>
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>
    }
}