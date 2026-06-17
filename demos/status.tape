Source demos/_settings.tape

Output demos/status.gif

Source demos/_cx_setup.tape
Type `cx bootstrap --force 2>/dev/null && cx create -n science python=3.12 scipy -y -q 2>/dev/null`
Enter
Wait /\$/
Type "clear"
Enter
Wait /\$/
Show

Type "# cx status shows what's installed"
Enter
Sleep 500ms

Type@80ms "cx status"
Enter
Wait /\$/
Sleep 3s

Type "# cx info shows the full conda configuration"
Enter
Sleep 500ms

Type@80ms "cx info"
Enter
Wait /\$/
Sleep 3s

Type "# List environments"
Enter
Sleep 500ms

Type@80ms "cx env list"
Enter
Wait /\$/
Sleep 3s
