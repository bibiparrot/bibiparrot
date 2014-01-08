#!/usr/bin/env python

import unittest
import logging
import wxversion
wxversion.select('2.8-unicode')
import wx

from ...bibiparrot.UIElements.MainFrame import MainFrame
from ...bibiparrot.UIElements.MainMenu import MainMenu


from ...bibiparrot.Configurations.Configuration import log


class TestMainMenu(unittest.TestCase):
    def setUp(self):
        self.app = wx.App(False)
        self.MainFrame = MainFrame(None)
        self.MainMenu = MainMenu(self.MainFrame)
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