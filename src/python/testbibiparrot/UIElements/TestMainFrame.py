#!/usr/bin/env python

import unittest
import logging
import wx

from ...bibiparrot.UIElements.MainFrame import MainFrame

from ...bibiparrot.Configurations.Configuration import log
#
#
# class Frame(wx.Frame):
#     def __init__(self, title):
#         wx.Frame.__init__(self, None, title=title, pos=(150,150), size=(350,200))
#         self.Bind(wx.EVT_CLOSE, self.OnClose)
#
#         menuBar = wx.MenuBar()
#         menu = wx.Menu()
#         m_exit = menu.Append(wx.ID_EXIT, "E&xit\tAlt-X", "Close window and exit program.")
#         self.Bind(wx.EVT_MENU, self.OnClose, m_exit)
#         menuBar.Append(menu, "&File")
#         self.SetMenuBar(menuBar)
#
#         self.statusbar = self.CreateStatusBar()
#
#         panel = wx.Panel(self)
#         box = wx.BoxSizer(wx.VERTICAL)
#
#         m_text = wx.StaticText(panel, -1, "Hello World!")
#         m_text.SetFont(wx.Font(14, wx.SWISS, wx.NORMAL, wx.BOLD))
#         m_text.SetSize(m_text.GetBestSize())
#         box.Add(m_text, 0, wx.ALL, 10)
#
#         m_close = wx.Button(panel, wx.ID_CLOSE, "Close")
#         m_close.Bind(wx.EVT_BUTTON, self.OnClose)
#         box.Add(m_close, 0, wx.ALL, 10)
#
#         panel.SetSizer(box)
#         panel.Layout()
#
#     def OnClose(self, event):
#             dlg = wx.MessageDialog(self,  "Do you really want to close this application?",
#                                    "Confirm Exit", wx.OK|wx.CANCEL|wx.ICON_QUESTION)
#             result = dlg.ShowModal()
#             dlg.Destroy()
#             if result == wx.ID_OK:
#                  self.Destroy()

# def OnClose(top, event):
#     dlg = wx.MessageDialog(top, "Do you really want to close this application?", "Confirm Exit", wx.OK|wx.CANCEL|wx.ICON_QUESTION)
#     result = dlg.ShowModal()
#     dlg.Destroy()
#     if result == wx.ID_OK:
#          top.Destroy()

class TestMainFrame(unittest.TestCase):
    def setUp(self):
        self.app = wx.App(redirect=True)
        self.MainFrame = MainFrame(None)
        pass


    def test(self):
        log().debug("%s: val=%s", "s",  "s")
        menuBar = wx.MenuBar()
        menu = wx.Menu()
        m_exit = menu.Append(wx.ID_EXIT, "E&xit\tAlt-X", "Close window and exit program.")
        self.MainFrame.Bind(wx.EVT_MENU, None, m_exit)
        menuBar.Append(menu, "&File")
        self.MainFrame.SetMenuBar(menuBar)
        self.MainFrame.CreateStatusBar()

        menuBar = wx.MenuBar()
        menu = wx.Menu()
        m_exit = menu.Append(wx.ID_EXIT, "E&xit\tAlt-X", "Close window and exit program.")
        self.MainFrame.Bind(wx.EVT_MENU, None, m_exit)
        menuBar.Append(menu, "&File")
        self.MainFrame.SetMenuBar(menuBar)
        self.MainFrame.Show()

        # app = wx.App(redirect=True)
        # top = Frame("Hello World")
        # top.Show()
        # app.MainLoop()

        pass

    def tearDown(self):
        self.app.MainLoop()
        pass

if __name__ == '__main__':
    unittest.main()