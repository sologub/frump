import React = require("react");

interface AppHeaderProps{

}

export class AppHeader extends React.Component<AppHeaderProps, {}>{

    render() {
        return <nav>
            <div className="nav-wrapper blue-grey darken-4">
                <a href="/" className="brand-logo left">&nbsp;Frump</a>
                <ul id="nav-mobile" className="right ">
                    <li><a href="#">Settings</a></li>
                </ul>
            </div>
        </nav>;
    }
}