#!/usr/bin/env python



import sys, os , time, inspect, imp, platform, logging
import wx


if sys.version_info < (2, 6):
    import warnings
    warnings.warn("This BibiParrot Version is Unsupported on Python Versions Older Than 2.6", ImportWarning)

log = logging.getLogger(__package__)

class BibiParrotFrame(wx.Frame):
    def __init__(self, parent, title):
        wx.Frame.__init__(self, parent, title=title, size=(200,100))
        self.control = wx.TextCtrl(self, style=wx.TE_MULTILINE)
        self.Show(True)

app = wx.App(False)  # Create a new app, don't redirect stdout/stderr to a window.
frame = BibiParrotFrame(None, "Hello World") # A Frame is a top-level window.
app.MainLoop()

