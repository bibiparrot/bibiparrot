#!/usr/bin/env python

from ...bibiparrot.Constants.constants import __required_wx_version__

import unittest
import sys, os
import logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
# import wx.tools.img2py

from ...bibiparrot.UIElements.MainFrame import MainFrame
from ...bibiparrot.UIElements.MainMenu import MainMenu
from ...bibiparrot.UIElements.MainToolbar import MainToolbar
from ...bibiparrot.UIElements.Editor import Editor
from ...bibiparrot.Configurations.Configuration import log


class TestEditor(unittest.TestCase):
    def setUp(self):
        pass

    def test(self):
        # fimg = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389875652_format-text-strikethrough.png"
        # fpy = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389875652_format-text-strikethrough.png.py"
        # fimg = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389894439_format-text-strikethrough.png"
        # fpy = "/Users/shi/Project/github_bibiparrot/bibiparrot/image/themes/default/1389894439_format-text-strikethrough.png.py"
        # wx.tools.img2py.img2py(fimg, fpy)

        self.app = wx.App(False)
        mainFrame = PyAUIFrame(None, title="wx.aui wxPython Demo", size=(750, 590))
        # self.Panel = wx.Panel(self.MainFrame)
        editor = Editor(mainFrame)
        mainMenu = MainMenu(mainFrame)
        mainToolbar = MainToolbar(mainFrame)
        # nullPanel = wx.Panel(mainFrame)
        mainFrame.uiman = wx.aui.AuiManager()

        # mainFrame.uiman.AddPane(nullPanel,wx.aui.AuiPaneInfo().CenterPane())
        mainFrame.uiman.AddPane(mainToolbar, wx.aui.AuiPaneInfo().
                      Name("Toolbar").Caption("Toolbar").
                      ToolbarPane().Top().Row(1).Position(1).
                      LeftDockable(False).RightDockable(False))
        mainFrame.uiman.AddPane(editor, wx.aui.AuiPaneInfo().CenterPane().CaptionVisible(True).Caption(editor.element.Title).CloseButton(False))

        mainFrame.uiman.SetManagedWindow(mainFrame)
        mainFrame.uiman.Update()
        mainFrame.Show()
        self.app.MainLoop()

        pass

    # def test(self):
    #     try:
    #         self.app = wx.App(False)
    #         frame = PyAUIFrame(None, title="wx.aui wxPython Demo", size=(750, 590))
    #         nullPanel = wx.Panel(frame)
    #         frame.uiman = wx.aui.AuiManager()
    #         frame.uiman.SetManagedWindow(frame)
    #         frame.uiman.AddPane(nullPanel,wx.aui.AuiPaneInfo().CenterPane())
    #         frame.uiman.Update()
    #         frame.Show()
    #         self.app.MainLoop()
    #     except:
    #         import traceback
    #         xc = traceback.format_exception(*sys.exc_info())
    #         wx.MessageBox(''.join(xc))
    #     pass

    def tearDown(self):
        pass

class PyAUIFrame(wx.Frame):

    def __init__(self, parent, id=-1, title="", pos=wx.DefaultPosition,
                 size=wx.DefaultSize, style=wx.DEFAULT_FRAME_STYLE |
                                            wx.SUNKEN_BORDER |
                                            wx.CLIP_CHILDREN):

        wx.Frame.__init__(self, parent, id, title, pos, size, style)

        # tell FrameManager to manage this frame
        # self._mgr = wx.aui.AuiManager()
        # self._mgr.SetManagedWindow(self)
        # self._mgr.Update()


if __name__ == '__main__':

        unittest.main()
        #
        # app = wx.App(False)
        # frame = MainFrame(None)
        # nullPanel = wx.Panel(frame)
        # frame.uiman = wx.aui.AuiManager()
        # frame.uiman.SetManagedWindow(frame)
        # frame.uiman.AddPane(nullPanel,wx.aui.AuiPaneInfo().CenterPane())
        # frame.uiman.Update()
        # frame.Show()
        # app.MainLoop()


        # app = wx.App(False)
        # frame = PyAUIFrame(None, title="wx.aui wxPython Demo", size=(750, 590))
        # frame.uiman = wx.aui.AuiManager()
        # nullPanel = wx.Panel(frame)
        # frame.uiman.AddPane(nullPanel,wx.aui.AuiPaneInfo().CenterPane())
        # frame.uiman.SetManagedWindow(frame)
        # frame.uiman.Update()
        # # frame._mgr.Update()
        # frame.Show()
        # app.MainLoop()

