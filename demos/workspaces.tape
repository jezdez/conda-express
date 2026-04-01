Source demos/_settings.tape

Output demos/workspaces.gif
Output demos/workspaces.mp4

Hide
Type `eval "$(pixi shell-hook -s bash -e default)"`
Enter
Wait /\)/
Type `export PS1="$DEMO_PS1" && mkdir -p /tmp/.demo-home && echo 'PS1="$ "' > /tmp/.demo-home/.bash_profile && export HOME=/tmp/.demo-home`
Enter
Wait /\$/
Type `export PATH="$HOME/.cx/condabin:$PATH" && cx bootstrap --force 2>/dev/null`
Enter
Wait /\$/
Type "rm -rf /tmp/cx-ws-demo && mkdir -p /tmp/cx-ws-demo && cd /tmp/cx-ws-demo"
Enter
Wait /\$/
Type "clear"
Enter
Wait /\$/
Show

Type "# conda-workspaces ships with every cx install"
Enter
Sleep 500ms

Type "# Initialize a project workspace"
Enter
Sleep 500ms

Type@80ms "cx workspace init --name my-project"
Enter
Wait /\$/
Sleep 1s

Type@80ms `cx workspace add "python>=3.12" numpy`
Enter
Wait /\$/
Sleep 1s

Type "# Add a task"
Enter
Sleep 500ms

Type@80ms `cx task add check 'python -c "import numpy; print(numpy.__version__)"'`
Enter
Wait /\$/
Sleep 1s

Type@80ms "bat conda.toml"
Enter
Wait /\$/
Sleep 3s

Type "# Install the workspace environment"
Enter
Sleep 500ms

Type@80ms "cx workspace install"
Enter
Wait /\$/
Sleep 2s

Type "# Run the task"
Enter
Sleep 500ms

Type@80ms "cx task run check"
Enter
Wait /\$/
Sleep 3s

Type "# Workspaces and tasks — built into every cx install"
Enter
Sleep 3s
