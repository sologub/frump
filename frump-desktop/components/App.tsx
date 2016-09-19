import React = require("react");
import {FrumpParser, Frump} from "../modules/Frump";

import {AppHeader} from "./AppHeader"
import {AppContent} from "./AppContent"

interface AppState{
    frump: Frump
}

export class App extends React.Component<{}, AppState>{

    constructor(props) {
        super(props);
        
        var parser = new FrumpParser();
        this.state = {
            frump: parser.Parse("C:/temp/frump.md")
        };

        // this.play = this.play.bind(this);
        // this.stop = this.stop.bind(this);
        // this.next = this.next.bind(this);
        // this.prev = this.prev.bind(this);
    }

    taskDetails(id: number) {
        var task = this.state.frump.Tasks.find(x => x.Id == id);


    }

    refresh() {
        
    }
    
    render() {
        return <div className="container-fluid">
            <AppHeader />
            <AppContent frump={this.state.frump}/>
        </div>
    }

}