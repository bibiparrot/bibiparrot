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
from ...bibiparrot.UIElements.Repeater import Repeater
from ...bibiparrot.Configurations.Configuration import log


class TestRepeater(unittest.TestCase):
    def setUp(self):

        self.app = wx.App(False)
        mainframe = MainFrame(None)
        # self.Panel = wx.Panel(mainframe)
        maintabs = MainTabs(mainframe)
        editor = Editor(mainframe)
        mainmenu = MainMenu(mainframe)
        maintoolbar = MainToolbar(mainframe)
        mainstatusbar = MainStatusbar(mainframe)
        repeater = Repeater(mainframe)
        mainframe.uiman = wx.aui.AuiManager()
        mainframe.uiman.SetManagedWindow(mainframe)
        mainframe.uiman.AddPane(maintoolbar, wx.aui.AuiPaneInfo().
                      Name("MainToolbar").Caption("MainToolbar").
                      ToolbarPane().Top().Row(1).Position(1).
                      LeftDockable(False).RightDockable(False).CloseButton(False))
        mainframe.uiman.AddPane(maintabs, wx.aui.AuiPaneInfo().Left().Layer(1).CaptionVisible(True).Caption(maintabs.element.Title).CloseButton(False))
        mainframe.uiman.AddPane(repeater, wx.aui.AuiPaneInfo().Top().CaptionVisible(True).Caption(repeater.element.Title).CloseButton(False))
        mainframe.uiman.AddPane(editor, wx.aui.AuiPaneInfo().CenterPane().CaptionVisible(True).Caption(editor.element.Title).CloseButton(False))

        mainframe.uiman.AddPane(mainstatusbar, wx.aui.AuiPaneInfo().
                      Name("MainStatusbar").Caption("MainStatusbar").
                      ToolbarPane().Bottom().Row(1).Position(1).
                      LeftDockable(False).RightDockable(False).CloseButton(False))
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
    import cProfile as profile
    profile.run('unittest.main()',sort=1)