################################################################################
# Name     : SelfControls.py                                                   #
# Brief    : Self Defined Controls                                             #
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
import wx.calendar
import  wx.gizmos

from ...bibiparrot.UIElements.UIElement import UIElement


from ...bibiparrot.Configurations.configurations import *

class SearchCtrl(wx.SearchCtrl):
    maxSearches = 5
    def __init__(self, parent, id=-1, value="",
                 pos=wx.DefaultPosition, size=wx.DefaultSize, style=0,
                 doSearch=None):
        style |= wx.TE_PROCESS_ENTER
        wx.SearchCtrl.__init__(self, parent, id, value, pos, size, style)
        self.Bind(wx.EVT_TEXT_ENTER, self.OnTextEntered)
        self.Bind(wx.EVT_MENU_RANGE, self.OnMenuItem, id=1, id2=self.maxSearches)
        self.doSearch = doSearch
        self.searches = []

    def OnTextEntered(self, evt):
        text = self.GetValue()
        if self.doSearch(text):
            self.searches.append(text)
            if len(self.searches) > self.maxSearches:
                del self.searches[0]
            self.SetMenu(self.MakeMenu())
        self.SetValue("")

    def OnMenuItem(self, evt):
        text = self.searches[evt.GetId()-1]
        self.doSearch(text)

    def MakeMenu(self):
        menu = wx.Menu()
        item = menu.Append(-1, "Recent Searches")
        item.Enable(False)
        for idx, txt in enumerate(self.searches):
            menu.Append(1+idx, txt)
        return menu


class LEDNumberCtrl(wx.gizmos.LEDNumberCtrl):
    def __init__(self, parent, *args, **kwargs):
        wx.gizmos.LEDNumberCtrl.__init__(self, parent,
                           -1,size = (280, 50),
                           style=wx.gizmos.LED_ALIGN_CENTER,
                           *args, **kwargs)


class VolumeCtrl(wx.Slider):
    def __init__(self, parent, *args, **kwargs):
        wx.Slider.__init__(self, parent, *args, **kwargs)

#
# class TestPanel(wx.Panel):
#     def __init__(self, parent, ID, log):
#         wx.Panel.__init__(self, parent, ID)
#         self.log = log
#
#         cal = wx.calendar.CalendarCtrl(self, -1, wx.DateTime_Now(), pos = (25,50),
#                              style = wx.calendar.CAL_SHOW_HOLIDAYS
#                              | wx.calendar.CAL_SUNDAY_FIRST
#                              | wx.calendar.CAL_SEQUENTIAL_MONTH_SELECTION
#                              )
#         self.cal = cal
#         self.Bind(wx.calendar.EVT_CALENDAR, self.OnCalSelected, id=cal.GetId())
#
#         # Set up control to display a set of holidays:
#         self.Bind(wx.calendar.EVT_CALENDAR_MONTH, self.OnChangeMonth, cal)
#         self.holidays = [(1,1), (10,31), (12,25) ]    # (these don't move around)
#         self.OnChangeMonth()
#
#         cal2 = wx.calendar.CalendarCtrl(self, -1, wx.DateTime_Now(), pos = (325,50))
#         self.Bind(wx.calendar.EVT_CALENDAR_SEL_CHANGED,
#                   self.OnCalSelChanged, cal2)
#
#     def OnCalSelected(self, evt):
#         self.log.write('OnCalSelected: %s\n' % evt.GetDate())
#
#     def OnChangeMonth(self, evt=None):
#         cur_month = self.cal.GetDate().GetMonth() + 1   # convert wxDateTime 0-11 => 1-12
#         for month, day in self.holidays:
#             if month == cur_month:
#                 self.cal.SetHoliday(day)
#         if cur_month == 8:
#             attr = wx.calendar.CalendarDateAttr(border=wx.calendar.CAL_BORDER_SQUARE,
#                                                 colBorder="blue")
#             self.cal.SetAttr(14, attr)
#         else:
#             self.cal.ResetAttr(14)
#
#     def OnCalSelChanged(self, evt):
#         cal = evt.GetEventObject()
#         self.log.write("OnCalSelChanged:\n\t%s: %s\n\t%s: %s\n\t%s: %s\n\t" %
#                        ("EventObject", cal,
#                         "Date       ", cal.GetDate(),
#                         "Ticks      ", cal.GetDate().GetTicks(),
#                         ))

selfctrls['EditSearchCtrl'] = SearchCtrl
selfctrls['LEDNumberCtrl'] = LEDNumberCtrl
selfctrls['MediaVolumeCtrl'] = VolumeCtrl

#
# class TestPanel(wx.Panel):
#     def __init__(self, parent, log):
#         self.log = log
#         wx.Panel.__init__(self, parent, -1)
#
#         self.ticker = Ticker(self)
#
#         #       Controls for ...controlling... the ticker.
#         self.txt = wx.TextCtrl(self, value="I am a scrolling ticker!!!!", size=(200,-1))
#         wx.CallAfter(self.txt.SetInsertionPoint, 0)
#         txtl = wx.StaticText(self, label="Ticker text:")
#         fgb = csel.ColourSelect(self, -1, colour=self.ticker.GetForegroundColour())
#         fgl = wx.StaticText(self, label="Foreground Color:")
#         bgb = csel.ColourSelect(self, -1, colour=self.ticker.GetBackgroundColour())
#         bgl = wx.StaticText(self, label="Background Color:")
#         fontb = wx.Button(self, label="Change")
#         self.fontl = wx.StaticText(self)
#         dirb = wx.Button(self, label="Switch")
#         self.dirl = wx.StaticText(self)
#         fpsl = wx.StaticText(self, label="Frames per Second:")
#         fps = wx.Slider(self, value=self.ticker.GetFPS(), minValue=1, maxValue=100,
#                         size=(150,-1),
#                         style=wx.SL_HORIZONTAL|wx.SL_AUTOTICKS|wx.SL_LABELS)
#         fps.SetTickFreq(5)
#         ppfl = wx.StaticText(self, label="Pixels per frame:")
#         ppf = wx.Slider(self, value=self.ticker.GetPPF(), minValue=1, maxValue=10,
#                         size=(150,-1),
#                         style=wx.SL_HORIZONTAL|wx.SL_AUTOTICKS|wx.SL_LABELS)
#
#         #       Do layout
#         sz = wx.FlexGridSizer(cols=2, hgap=4, vgap=4)
#
#         sz.Add(txtl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(self.txt, flag=wx.ALIGN_CENTER_VERTICAL)
#
#         sz.Add(fgl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(fgb, flag=wx.ALIGN_CENTER_VERTICAL)
#
#         sz.Add(bgl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(bgb, flag=wx.ALIGN_CENTER_VERTICAL)
#
#         sz.Add(self.fontl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(fontb, flag=wx.ALIGN_CENTER_VERTICAL)
#
#         sz.Add(self.dirl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(dirb, flag=wx.ALIGN_CENTER_VERTICAL)
#
#         sz.Add(fpsl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(fps, flag=wx.ALIGN_CENTER_VERTICAL|wx.ALIGN_LEFT)
#
#         sz.Add(ppfl, flag=wx.ALIGN_CENTER_VERTICAL)
#         sz.Add(ppf, flag=wx.ALIGN_CENTER_VERTICAL|wx.ALIGN_LEFT)
#
#         sz2 = wx.BoxSizer(wx.VERTICAL)
#         sz2.Add(self.ticker, flag=wx.EXPAND|wx.ALL, border=5)
#         sz2.Add(sz, flag=wx.EXPAND|wx.ALL, proportion=1, border=25)
#         self.SetSizer(sz2)
#         sz2.SetSizeHints(self)
#
#         #       Bind events
#         self.Bind(wx.EVT_BUTTON, self.OnChangeTickDirection, dirb)
#         self.Bind(wx.EVT_BUTTON, self.OnChangeTickFont, fontb)
#         self.Bind(wx.EVT_TEXT, self.OnText, self.txt)
#         self.Bind(csel.EVT_COLOURSELECT, self.ChangeTickFGColor, fgb)
#         self.Bind(csel.EVT_COLOURSELECT, self.ChangeTickBGColor, bgb)
#         self.Bind(wx.EVT_SCROLL, self.ChangeFPS, fps)
#         self.Bind(wx.EVT_SCROLL, self.ChangePPF, ppf)
#
#         #       Set defaults
#         self.SetTickDirection("rtl")
#         self.SetTickFont(self.ticker.GetFont())
#         self.ticker.SetText(self.txt.GetValue())
#
#
#     def SetTickFont(self, font):
#         """Sets ticker font, updates label"""
#         self.ticker.SetFont(font)
#         self.fontl.SetLabel("Font: %s"%(self.ticker.GetFont().GetFaceName()))
#         self.Layout()
#
#
#     def OnChangeTickFont(self, evt):
#         fd = wx.FontData()
#         fd.EnableEffects(False)
#         fd.SetInitialFont(self.ticker.GetFont())
#         dlg = wx.FontDialog(wx.GetTopLevelParent(self), fd)
#         if dlg.ShowModal() == wx.ID_OK:
#             data = dlg.GetFontData()
#             self.SetTickFont(data.GetChosenFont())
#
#
#     def SetTickDirection(self, dir):
#         """Sets tick direction, updates label"""
#         self.ticker.SetDirection(dir)
#         self.dirl.SetLabel("Direction: %s"%(self.ticker.GetDirection()))
#
#
#     def OnChangeTickDirection(self, dir):
#         if self.ticker.GetDirection() == "rtl":
#             self.SetTickDirection("ltr")
#         else:
#             self.SetTickDirection("rtl")
#
#
#     def OnText(self, evt):
#         """Live update of the ticker text"""
#         self.ticker.SetText(self.txt.GetValue())
#
#     def ChangeTickFGColor(self, evt):
#         self.ticker.SetForegroundColour(evt.GetValue())
#
#     def ChangeTickBGColor(self, evt):
#         self.ticker.SetBackgroundColour(evt.GetValue())
#
#     def ChangeFPS(self, evt):
#         self.ticker.SetFPS(evt.GetPosition())
#
#     def ChangePPF(self, evt):
#         self.ticker.SetPPF(evt.GetPosition())
#
#
#     def ShutdownDemo(self):
#         self.ticker.Stop()

