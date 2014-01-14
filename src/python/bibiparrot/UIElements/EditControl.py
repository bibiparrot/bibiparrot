################################################################################
# Name     : EditorControl.py                                                  #
# Brief    : Define the base functions to control a rtf file                   #
#                                                                              #
# Url      :                                                                   #
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
from ...bibiparrot.Configurations.configurations import *



class EditControl(wx.richtext.RichTextCtrl):
    def __init__(self, parent, *args, **kwargs):
        wx.richtext.RichTextCtrl.__init__(self, parent, style=wx.VSCROLL|wx.HSCROLL|wx.NO_BORDER, *args, **kwargs)
        wx.CallAfter(self.SetFocus)

        self.Freeze()
        self.BeginSuppressUndo()

        self.BeginParagraphSpacing(0, 20)

        self.BeginAlignment(wx.richtext.TEXT_ALIGNMENT_CENTRE)
        self.BeginBold()

        self.BeginFontSize(14)
        self.WriteText("Welcome to wxRichTextCtrl, a wxWidgets control for editing and presenting styled text and images")
        self.EndFontSize()
        self.Newline()
        self.EndSuppressUndo()
        self.Thaw()
        pass