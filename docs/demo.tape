Output docs/demo.gif

Set Shell zsh
Set PlaybackSpeed 0.5 # Make output 2 times slower
Set WindowBar Colorful
Set Theme nord
Set Padding 20
Set FontSize 20
Set Width 1000
Set Margin 10
Set MarginFill "#674EFF"
Set BorderRadius 10

# Setup
Hide
Type "alias friday=./target/release/friday && clear"
Enter
Sleep 2s
Show

Type "friday help"
Enter
Sleep 1s

Type "export FRIDAY_FILE=friday.md"
Enter
Enter
Type "friday show"
Enter
Sleep 1s

Type "friday add 'something else'"
Enter
Enter
Sleep 1s

Type "by default all tasks are stored in a Markdown file"
Ctrl+C
Type "bat friday.md"
Enter
Sleep 5s

Hide
Type "rm friday.md"
Enter