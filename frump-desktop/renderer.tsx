// This file is required by the index.html file and will
// be executed in the renderer process for that window.
// All of the Node.js APIs are available in this process.

import React = require("react");
import ReactDom = require("react-dom");
import { browserHistory, Router, Route, Link } from "react-router";

import {App} from "./components/App";

import {TaskDetails} from "./components/Frump/TaskDetails";

export class Main extends React.Component<{}, {}>{
    render() {
        return <Router history={withExampleBasename(browserHistory, __dirname) }>
            <Route path="/" component={App}>
                <Route path="category/:category" components={{ content: Category, sidebar: CategorySidebar }}>
                    <Route path=":item" component={Item} />
                </Route>
            </Route>
        </Router>
    }
}

ReactDom.render(<App />, document.querySelector("main"));
