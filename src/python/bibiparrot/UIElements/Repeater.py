################################################################################
# Name     : Repeater.py                                                       #
# Brief    : The Panel to hold repeater controller.                            #
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
import wx.media
import time
import wx.richtext

from ..Configurations import configurations
from ..Constants import constants
from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.UIElements.EditControl import EditControl
from ...bibiparrot.UIElements.MainMenu import MainMenu
from ...bibiparrot.UIElements.MainToolbar import  RepeaterToolbar

from ...bibiparrot.Configurations.configurations import *
import EventIDs
import Images


###
##  Used for media progress control.
#

class MediaSlider(wx.Slider):
    def __init__(self, parent,  *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("MediaSlider")
        wx.Slider.__init__(self, parent,  size = self.element.Size,
                           *args, **kwargs)


class PlayerInfo(object):
    __slots__= ['Volume', 'SeekPoint']
    def __init__(self):
        self.Volume = 0
        self.SeekPoint = 0



# class MediaPlayer(wx.Panel):
#     def __init__(self, parent,  *args, **kwargs):
#         wx.Panel.__init__(self, parent,  *args, **kwargs)

class MediaPlayer(wx.MiniFrame):
    TYPE_WX = 0
    TYPE_VLC = 1
    def __init__(self, parent,  *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("MediaPlayer")
        self.info = PlayerInfo()
        wx.MiniFrame.__init__(self, parent, style=wx.DEFAULT_FRAME_STYLE, *args, **kwargs)

        ### None, 0 = wx.media.MediaCtrl, 1 = VLCMediaPlayCtrl###
        self.ctrlType = None
        ### Define controller  ###
        try:
            ### First, we choose portable VLC Media Player  ###
            from ..MediaElements.MediaPlayControl import VLCMediaPlayCtrl
            self.ctrl = VLCMediaPlayCtrl(self.GetHandle())
            self.ctrl.load()
            self.ctrlType = MediaPlayer.TYPE_VLC
        except (ImportError, ValueError) as err:
            ###
            print err
            ### Second, if VLC is not available, we use original wx.media.MediaCtrl ###
            try:
                backend = ""
                if 'wxMSW' in wx.PlatformInfo:
                    # the default backend doesn't always send the EVT_MEDIA_LOADED
                    # event which we depend upon, so use a different backend by
                    # default for this demo.
                    backend = wx.media.MEDIABACKEND_QUICKTIME

                self.ctrl = wx.media.PreMediaCtrl()
                ok = self.ctrl.Create(self, style=wx.SIMPLE_BORDER,
                                    szBackend=backend)
                self.ctrlType = MediaPlayer.TYPE_WX
                if not ok:
                    raise NotImplementedError
                self.ctrl.PostCreate(self.ctrl)
            except NotImplementedError:
                self.Destroy()

        ### define necessary event bindings ###
        self.Bind(wx.EVT_CLOSE, self.Close)


    def Close(self, evt):
        self.Stop()

    def Open(self):
        dlg = wx.FileDialog(self, message="Choose a media file",
                            defaultDir=os.getcwd(), defaultFile="",
                            style=wx.OPEN | wx.CHANGE_DIR )
        if dlg.ShowModal() == wx.ID_OK:
            path = dlg.GetPath()
            self.LoadFile(path)
        dlg.Destroy()


    def LoadFile(self, path):
        # self.playBtn.Disable()
        ### TODO: enable play or display button ###
        ### TODO: set the volume of slides  ###
        ### TODO: set slider range ###
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            self.ctrl.open(path)
            vsize = self.ctrl.getVideoSize()
            self.SetSize(vsize)
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            if not self.ctrl.Load(path):
                wx.MessageBox("Unable to load %s: Unsupported format?" % path,
                              "ERROR",
                              wx.ICON_ERROR | wx.OK)
            else:
                self.ctrl.SetInitialSize()
            # self.slider.SetRange(0, self.mc.Length())
        ## Show my self. ##

    def Play(self):
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            self.ctrl.play()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            if not self.ctrl.Play():
                wx.MessageBox("Unable to Play media : Unsupported format?",
                              "ERROR",
                              wx.ICON_ERROR | wx.OK)
            else:
                self.ctrl.SetInitialSize()
                # self.GetSizer().Layout()
                # self.slider.SetRange(0, self.mc.Length())
        self.Show(True)

    def Pause(self):
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            self.ctrl.pause()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            self.ctrl.Pause()

    def Stop(self):
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            self.ctrl.stop()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            self.ctrl.Stop()
        self.Show(False)

    def Seek(self, pos=None):
        # offset = self.slider.GetValue()
        if pos is None:
            pos = self.info.SeekPoint
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            self.ctrl.seek(pos)
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            self.ctrl.Seek(pos)

    def GetCurrTime(self):
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            return self.ctrl.getMediaCurrTime()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            return self.ctrl.Tell()

    def GetLength(self):
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            return self.ctrl.getMediaLength()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            return self.ctrl.Length()

class Repeater(wx.Panel):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("Repeater")
        wx.Panel.__init__(self, parent, size = self.element.Size, *args, **kwargs)
        self.RepeaterToolbar = RepeaterToolbar(self)
        self.MediaSlider = MediaSlider(self)
        self.MediaPlayer = MediaPlayer(self)
        self.uiman = wx.aui.AuiManager()
        self.uiman.SetManagedWindow(self)
        self.timer = wx.Timer(self)
        self.timer.Start(200)
        self.Bind(wx.EVT_TIMER, self.OnTimer)
        # self.uiman.AddPane(self.control, wx.aui.AuiPaneInfo().
        #           CenterPane())
        self.uiman.AddPane(self.RepeaterToolbar, wx.aui.AuiPaneInfo().
                  Name("RepeaterToolbar").Caption("RepeaterToolbar").
                  ToolbarPane().Top().Row(1).Position(1).
                  LeftDockable(False).RightDockable(False).CloseButton(False))
        self.uiman.AddPane(self.MediaSlider, wx.aui.AuiPaneInfo().
                  Name("MediaSlider").Caption("MediaSlider").
                  ToolbarPane().Top().Row(1).Position(2).
                  LeftDockable(False).RightDockable(False).CloseButton(False))
        self.uiman.Update()
        self.binds()


    def binds(self):
        self.MediaSlider.Bind(wx.EVT_SLIDER, self.OnSeek, self.MediaSlider)

        for id in self.RepeaterToolbar.binds.keys():
            (toolbar, item) = self.RepeaterToolbar.binds[id]
            # print "On%s"%(toolbar.Name)
            handler = getattr(self, "On%s"%(toolbar.Name))
            # print handler
            # print item
            # print
            self.Bind(wx.EVT_TOOL, handler, item)
            if toolbar.needsUpdate():
                updatehandler = getattr(self, "OnUpdate%s"%(toolbar.Name), handler)
                # print updatehandler
                # print "OnUpdate%s"%(toolbar.Name)
                self.Bind(wx.EVT_UPDATE_UI, updatehandler, item)




    def OnMediaOpen(self, evt):
        self.MediaPlayer.Open()
        if LOGWIRE:
            log().debug("%s: Length=%s", funcname(), self.MediaPlayer.GetLength())
        self.MediaSlider.SetRange(0, self.MediaPlayer.GetLength())

    def OnMediaLoaded(self, evt):
        # self.playBtn.Enable()
        pass

    def OnMediaPlayAndPause(self, evt):
        self.MediaPlayer.Play()
        (wxId, elem) = EventIDs.getElementbyName('MediaPlayAndPause')
        self.RepeaterToolbar.SetToolNormalBitmap(wxId, Images.bitmap('media_pause_32x32'))

    def OnPauseAndResume(self, evt):
        pass

    def OnMediaStop(self, evt):
        self.MediaPlayer.Stop()

    def OnMediaBegin(self, evt):
        print 'OnMediaBegin'
        pass

    def OnMediaEnd(self, evt):
        print 'OnMediaEnd'
        pass

    def OnMediaVolume(self, evt):
        print 'OnMediaVolume'
        pass

    def OnSeek(self, evt):
        self.MediaPlayer.info.SeekPoint = self.MediaSlider.GetValue()
        self.MediaPlayer.Seek()

    def OnTimer(self, evt):
        offset = self.MediaPlayer.GetCurrTime()
        self.MediaSlider.SetValue(offset)
        # ct = time.gmtime(offset/1000)
        # print_time = (ct[0], ct[1], ct[2], ct[3], ct[4], ct[5], ct[6], ct[7], -1)
        # self.MediaSlider.SetLabel(time.strftime("%H:%M:%S", ct))
        # self.st_size.SetLabel('size: %s' % self.mc.GetBestSize())
        # self.st_len.SetLabel('length: %d seconds' % (self.mc.Length()/1000))
        # self.st_pos.SetLabel('position: %d' % offset)

