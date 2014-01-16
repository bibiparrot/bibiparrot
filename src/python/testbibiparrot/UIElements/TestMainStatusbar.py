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
from ...bibiparrot.UIElements.MainStatusbar import MainStatusbar
from ...bibiparrot.Configurations.Configuration import log


class TestMainStatusbar(unittest.TestCase):
    def setUp(self):

        # fimg = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389875652_format-text-strikethrough.png"
        # fpy = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389875652_format-text-strikethrough.png.py"
        # fimg = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389894439_format-text-strikethrough.png"
        # fpy = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389894439_format-text-strikethrough.png.py"
        # wx.tools.img2py.img2py(fimg, fpy)

        self.app = wx.App(False)
        self.MainFrame = MainFrame(None)
        # self.Panel = wx.Panel(self.MainFrame)
        self.MainTabs = MainTabs(self.MainFrame)
        self.Editor = Editor(self.MainFrame)
        self.MainMenu = MainMenu(self.MainFrame)
        self.MainToolbar = MainToolbar(self.MainFrame)
        self.MainStatusbar = MainStatusbar(self.MainFrame)
        self.uiman = wx.aui.AuiManager()
        self.uiman.SetManagedWindow(self.MainFrame)
        self.uiman.AddPane(self.MainToolbar, wx.aui.AuiPaneInfo().
                      Name("MainToolbar").Caption("MainToolbar").
                      ToolbarPane().Top().Row(1).Position(1).
                      LeftDockable(False).RightDockable(False).CloseButton(False))
        self.uiman.AddPane(self.Editor, wx.aui.AuiPaneInfo().CenterPane().CaptionVisible(True).Caption(self.Editor.element.Title).CloseButton(False))
        self.uiman.AddPane(self.MainTabs, wx.aui.AuiPaneInfo().CaptionVisible(True).Caption(self.MainTabs.element.Title).CloseButton(False))
        self.uiman.AddPane(self.MainStatusbar, wx.aui.AuiPaneInfo().
                      Name("MainStatusbar").Caption("MainStatusbar").
                      ToolbarPane().Bottom().Row(1).Position(1).
                      LeftDockable(False).RightDockable(False).CloseButton(False))
        self.uiman.Update()
        self.MainFrame.Show()

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