#!/usr/bin/env python

from ...bibiparrot.Constants.constants import __required_wx_version__

import unittest
import logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.tools.img2py

from ...bibiparrot.UIElements.MainFrame import MainFrame
from ...bibiparrot.UIElements.MainMenu import MainMenu
from ...bibiparrot.UIElements.MainToolbar import MainToolbar
from ...bibiparrot.UIElements.Editor import Editor
from ...bibiparrot.UIElements.MainTabs import MainTabs
from ...bibiparrot.Configurations.Configuration import log


class TestMainTabs(unittest.TestCase):
    def setUp(self):

        # fimg = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389875652_format-text-strikethrough.png"
        # fpy = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389875652_format-text-strikethrough.png.py"
        # fimg = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389894439_format-text-strikethrough.png"
        # fpy = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389894439_format-text-strikethrough.png.py"
        # wx.tools.img2py.img2py(fimg, fpy)

        self.app = wx.App(False)
        mainframe = MainFrame(None)
        # self.Panel = wx.Panel(mainframe)
        maintabs = MainTabs(mainframe)
        editor = Editor(mainframe)
        mainmenu = MainMenu(mainframe)
        maintoolbar = MainToolbar(mainframe)

        mainframe.uiman = wx.aui.AuiManager()
        mainframe.uiman.SetManagedWindow(mainframe)
        mainframe.uiman.AddPane(maintoolbar, wx.aui.AuiPaneInfo().
                      Name("Toolbar").Caption("Toolbar").
                      ToolbarPane().Top().Row(1).Position(1).
                      LeftDockable(False).RightDockable(False))
        mainframe.uiman.AddPane(editor, wx.aui.AuiPaneInfo().CenterPane().CaptionVisible(True).Caption(editor.element.Title).CloseButton(False))
        mainframe.uiman.AddPane(maintabs, wx.aui.AuiPaneInfo().CaptionVisible(True).Caption(maintabs.element.Title).CloseButton(False))

        mainframe.uiman.Update()
        mainframe.Show()

    def test(self):
        log().debug("%s: val=%s", "s",  "s")
        # self.app.MainLoop()
        # print __name__
        # print __file__
        # print globals()
        pass

    def tearDown(self):
        self.app.MainLoop()

if __name__ == '__main__':
    unittest.main()