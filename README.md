# Frump

Distributed tasks management tool based on Git and Markdown.

## Why?

Many task and project management solutions have good integration with Git,
but a problem is usually left unsolved: they keep all tasks and their flow
outside Git, so it is hard to have a full picture of project's status and past
development only by looking at Git log.  
The next unsolved problem is that while your code is in a distributed system,
your tasks are not, preventing a team form a truly distributed collaboration.  

This tool aims to solve these very problems: when you clone a repo that uses 
Frump then you get not only the code but also all current and past tasks 
with their full history.

## Why "frump"?

Because with git they make a lovely couple.

## How?

Frump aims to be simple and human readable, so Markdown is selected as the
underlying file format, a file (usually `frump.md`) is used to keep the state
of the current tasks and Git log is used as the database for past changes.

## Philosophy

The most important feature of Frump is that you can use it without any tool.
Knowing Frump's conventions is the only requirement of effectively using it.

This feature leads to the next one: Frump is as simple as possible, you can
open `frump.md` and easily read and edit all tasks, and you can use git log
to see all past changes and closed tasks (a task is considered closed when
is removed from `frump.md` file).  

And of course `git pull`, along with the new code, gets you new tasks and tasks
changes and `git push` pushes your task changes in the same commit with your
code.  
You can branch your tasks, merge them and delete, the same as with your code. 

## Format

The format of the `frump.md` file is Markdown with a few conventions.

### Header Section

At the beginning of the file optionally starts the _Header_ section which is
any well formatted Markdown text that does not contain a size 2 heading element
(`##` in Markdown) with text `Team` or `Tasks` inside.  
In the header it is recommended to have only the title of the project with a 
brief description, so you won't have to scroll too much down to see the tasks.

### Team Section

Then comes optionally the _Team_ section, which starts with a `## Team` size 2 
heading element, followed by a list of all team members with emails and roles:
```
## Team
* John Doe <john@example.com> - Project Manager
* Ivan Smith <smith@example.com> - Developer
```
The first team member in the team list is used as the default task assignee,
when not explicitly specified in task, so usually there is placed the leader
of the team.  

If your team is big then it is good practice to place this section at the end
of the file, as Frump does not impose any ordering of its sections.

### Tasks Section

The _Tasks_ section starts with a `## Tasks` heading element followed by any
number of task items. A task item starts with a level 3 heading which contains
the task's ID, type and subject. Then in a new line follow the optional body
and properties:
```
### Task 3 - Write docs
Describe in README.md Tasks.md's philosophy, format and usage scenarios.  
_Status: working, assigned to Ruslan_
```
Here the ID is `3`, type is `Task` and subject is `Write docs`. The ID must be
a unique positive integer number. The type of a task can be anything, just try
to use a small and consistent set of possible values, like `Task`, `Bug`,
`Issue`. The subject is required and is not a title, so it shouldn't be
capitalized.  

The _body_ is everything following the _heading_, and then start the
_properties_. A property starts with an underscore `_` followed by a property
name and value, separated by a `:` symbol and ends with another underscore.
The underscores are required because otherwise it won't be easy to
automatically distinguish them from the body.
