################################################################################
# Name     : SelfDialogs.py                                                    #
# Brief    : Self Defined Dialogs                                              #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.Frame-class.html              #
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
from ...bibiparrot.UIElements.MainToolbar import MainToolbar, EditorToolbar
import Images
from ...bibiparrot.Configurations.configurations import *
#
try:
    from agw import pybusyinfo as PBI
except ImportError: # if it's not there locally, try the wxPython lib.
    import wx.lib.agw.pybusyinfo as PBI

class BusyInfo():
    def OnButton(self, evt):
        evt.Skip()
        message = "Please wait 5 seconds, working..."
        busy = PBI.PyBusyInfo(message, parent=None, title="Really Busy",
                              icon=Images.Smiles.GetBitmap())

        wx.Yield()

        for indx in xrange(5):
            wx.MilliSleep(1000)

        del busy
