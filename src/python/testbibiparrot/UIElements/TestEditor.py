#!/usr/bin/env python

from ...bibiparrot.Constants.constants import __required_wx_version__

import unittest
import logging
import wxversion
wxversion.select(__required_wx_version__)
import wx


from ...bibiparrot.UIElements.MainFrame import MainFrame
from ...bibiparrot.UIElements.MainMenu import MainMenu
from ...bibiparrot.UIElements.MainToolbar import MainToolbar
from ...bibiparrot.UIElements.Editor import Editor
from ...bibiparrot.Configurations.Configuration import log


class TestEditor(unittest.TestCase):
    def setUp(self):
        self.app = wx.App(False)
        self.MainFrame = MainFrame(None)
        self.MainMenu = MainMenu(self.MainFrame)
        self.MainToolbar = MainToolbar(self.MainFrame)
        self.Editor = Editor(self.MainFrame)
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