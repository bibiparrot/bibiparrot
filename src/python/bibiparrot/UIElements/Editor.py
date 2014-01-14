################################################################################
# Name     : Editor.py                                                         #
# Brief    : #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.ToolBar-class.html            #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################



from ...bibiparrot.Constants.constants import __required_wx_version__

import sys, os , time, inspect, imp, platform, logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.grid
import wx.html
import wx.aui
import wx.richtext

from ..Configurations import configurations
from ..Constants import constants
from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.UIElements.EditControl import EditControl
from ...bibiparrot.UIElements.MainMenu import MainMenu
from ...bibiparrot.UIElements.MainToolbar import MainToolbar

from ...bibiparrot.Configurations.configurations import *
#
#
# class Editor(wx.Panel):
#     def __init__(self, parent):
#         wx.Panel.__init__(self, -1, size=(300,300))
#         # self._mgr = wx.aui.AuiManager()
#         # self._mgr.SetManagedWindow(self)
#         self.element = UIElement()
#         self.element.loadSect("Editor")
#         print "XXXX", self.GetSizeTuple
#         self.rtc = wx.richtext.RichTextCtrl(self, style=wx.VSCROLL|wx.HSCROLL|wx.NO_BORDER, size=(300,300))
#         # sizer = wx.BoxSizer(wx.VERTICAL)
#         # sizer.Add(self.rtc)
#         # self.SetSizer(sizer)
#
#         wx.CallAfter(self.rtc.SetFocus)
#
#         self.rtc.Freeze()
#         self.rtc.BeginSuppressUndo()
#
#         self.rtc.BeginParagraphSpacing(0, 20)
#
#         self.rtc.BeginAlignment(wx.richtext.TEXT_ALIGNMENT_CENTRE)
#         self.rtc.BeginBold()
#
#         self.rtc.BeginFontSize(14)
#         self.rtc.WriteText("Welcome to wxRichTextCtrl, a wxWidgets control for editing and presenting styled text and images")
#         self.rtc.EndFontSize()
#         self.rtc.Newline()
#         self.rtc.EndSuppressUndo()
#         self.rtc.Thaw()
#         pass



class Editor(wx.Panel, UIElement):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("Editor")
        wx.Panel.__init__(self, parent, size = self.element.Size, *args, **kwargs)
        self.control = EditControl(self, size = self.element.Size)
        self.Toolbar = MainToolbar(self)

        self._mgr = wx.aui.AuiManager()
        self._mgr.SetManagedWindow(self)
        self._mgr.AddPane(self.control, wx.aui.AuiPaneInfo().
                          CenterPane())
        self._mgr.AddPane(self.Toolbar, wx.aui.AuiPaneInfo().
                          Name("Toolbar").Caption("Toolbar").
                          ToolbarPane().Top().Row(1).Position(1).
                          LeftDockable(False).RightDockable(False))
        self._mgr.Update()
        pass