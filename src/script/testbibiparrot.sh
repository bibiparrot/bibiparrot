#!/bin/sh


#FILE=$(readlink -f "$0")
#HOME=$(dirname $FILE)
HOME="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $HOME/..
python -m python.testbibiparrot.Configurations.TestBibiException
python -m python.testbibiparrot.Configurations.TestConfiguration
#python -m python.testbibiparrot.UIElements.TestUIElement
#python -m python.testbibiparrot.UIElements.TestMainMenu
python -m python.testbibiparrot.UIElements.TestMainToolbar
#python -m python.testbibiparrot.UIElements.TestEditor
cd $HOME
