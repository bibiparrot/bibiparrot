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

from ..MediaElements.MediaPlayControl import MediaInfo, MediaType, MediaState

###
##  Used for media progress control.
#

class MediaSlider(wx.Slider):
    def __init__(self, parent,  *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("MediaSlider")
        wx.Slider.__init__(self, parent,  size = self.element.Size,
                           *args, **kwargs)

class MediaPlayer(wx.MiniFrame):
    TYPE_WX = 0
    TYPE_VLC = 1
    def __init__(self, parent,  *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("MediaPlayer")
        self.info = MediaInfo()
        wx.MiniFrame.__init__(self, parent, style=wx.DEFAULT_FRAME_STYLE|wx.STAY_ON_TOP, *args, **kwargs)

        ### None, 0 = wx.media.MediaCtrl, 1 = VLCMediaPlayCtrl###
        self.ctrlType = None
        ### Define controller  ###
        try:
            ### First, we choose portable VLC Media Player  ###
            from ..MediaElements.MediaPlayControl import VLCMediaPlayCtrl
            self.ctrl = VLCMediaPlayCtrl(self.GetHandle(), self.info)
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
            ### window xp will give problem without encoding as uri.###
            path = path2url(path)
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
        willShow = False
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            willShow = self.ctrl.play()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            willShow = self.ctrl.Play()
            if not willShow:
                wx.MessageBox("Unable to Play media : Unsupported format?",
                              "ERROR",
                              wx.ICON_ERROR | wx.OK)
            else:
                self.ctrl.SetInitialSize()
        ### if it is audio without video size ###
        if self.info.videosize ==(0,0):
            willShow = False
        self.Show(willShow)

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

    def Volume(self, vol=None):
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            if vol is None:
                return self.ctrl.getVolume()
            else:
                self.ctrl.setVolume(vol)
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            if vol is None:
                return self.ctrl.GetVolume()
            else:
                self.ctrl.SetVolume(vol)

    STATE_MAP = { wx.media.MEDIASTATE_PLAYING:MediaState.Playing,
                  wx.media.MEDIASTATE_PAUSED:MediaState.Paused,
                  wx.media.MEDIASTATE_STOPPED:MediaState.Stopped }

    ### this is not correctly realized, really needs more testing ###
    def GetState(self):
        ### update ###
        if self.ctrlType == MediaPlayer.TYPE_VLC:
            self.info.state = self.ctrl.getState()
        elif self.ctrlType == MediaPlayer.TYPE_WX:
            stat = self.STATE_MAP.get(self.ctrl.GetState(), None)
            if stat is not None:
                self.info.state = stat
        # if LOGWIRE:
        #     log().debug("%s: state=%s", funcname(), self.info.state)
        return self.info.state

class Repeater(wx.Panel):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("Repeater")
        wx.Panel.__init__(self, parent, size = self.element.Size, *args, **kwargs)
        self.RepeaterToolbar = RepeaterToolbar(self)
        self.VolumeSlider = self.RepeaterToolbar.ctrls.get('MediaVolumeCtrl')
        self.MediaSlider = MediaSlider(self)
        self.MediaPlayer = MediaPlayer(self)
        self.uiman = wx.aui.AuiManager()
        self.uiman.SetManagedWindow(self)
        self.timer = wx.Timer(self)
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
        self.VolumeSlider.Bind(wx.EVT_SLIDER, self.OnVolume, self.VolumeSlider)
        self.Bind(wx.EVT_TIMER, self.OnTimer)
        print "binds"
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
                self.Bind(wx.EVT_UPDATE_UI, updatehandler, item)
                if LOGWIRE:
                    log().debug("%s: Length=%s", funcname(), self.MediaPlayer.GetLength())

        ### start time after bindings ###
        self.timer.Start(200)


    def OnMediaOpenAndStop(self, evt):
        stat = self.MediaPlayer.GetState()
        if stat == MediaState.Playing or stat == MediaState.Paused:
            self.MediaPlayer.Stop()
        else:
            if stat == MediaState.Ended:
                self.MediaPlayer.Stop()
            self.MediaPlayer.Open()
            self.MediaSlider.SetRange(0, self.MediaPlayer.GetLength())
            self.VolumeSlider.SetValue(self.MediaPlayer.Volume())
            self.MediaPlayer.Play()

        ### Change the Icons
        # (wxId, item) = EventIDs.getElementbyName('MediaOpenAndStop')
        # bitmap = item.GetNormalBitmap()
        # self.RepeaterToolbar.SetToolNormalBitmap(wxId, item.GetClientData())
        # item.SetClientData(bitmap)

    def OnUpdateMediaOpenAndStop(self, evt):
        # evt.Check(self.rtc.IsSelectionBold())
        evtObj = evt.GetEventObject()
        wxId = evt.GetId()
        button = evtObj.FindById(wxId)
        shown = button.GetClientData()['Shown']
        stat = self.MediaPlayer.GetState()
        if stat == MediaState.Playing or stat == MediaState.Paused:
            if shown != 'IconMore':
                evtObj.SetToolNormalBitmap(wxId, button.GetClientData()['IconMore'])
                button.GetClientData()['Shown']='IconMore'
        else:
            if shown != 'Icon':
                evtObj.SetToolNormalBitmap(wxId, button.GetClientData()['Icon'])
                button.GetClientData()['Shown']='Icon'

    def OnMediaPlayAndPause(self, evt):
        stat = self.MediaPlayer.GetState()
        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.MediaPlayer.info.dump())
        if stat == MediaState.Playing:
            self.MediaPlayer.Pause()
        else:
            if stat == MediaState.Ended:
                self.MediaPlayer.Stop()
                # self.MediaPlayer.Seek(0)
            self.MediaPlayer.Play()


    def OnUpdateMediaPlayAndPause(self, evt):
        # if LOGWIRE:
        #     log().debug("%s: info=%s", funcname(), self.MediaPlayer.info.dump())
        evt.Enable(self.MediaPlayer.GetState() >= MediaState.Opened)
        evtObj = evt.GetEventObject()
        wxId = evt.GetId()
        button = evtObj.FindById(wxId)
        shown = button.GetClientData()['Shown']
        stat = self.MediaPlayer.GetState()
        if stat == MediaState.Playing:
            if shown != 'IconMore':
                evtObj.SetToolNormalBitmap(wxId, button.GetClientData()['IconMore'])
                button.GetClientData()['Shown']='IconMore'
        else:
            if shown != 'Icon':
                evtObj.SetToolNormalBitmap(wxId, button.GetClientData()['Icon'])
                button.GetClientData()['Shown']='Icon'
    # def OnUpdateMediaBegin(self, evt):
    #     evt.Enable(self.MediaPlayer)

    # def OnMediaStop(self, evt):
    #     self.MediaPlayer.Stop()

    def OnMediaBegin(self, evt):
        print 'OnMediaBegin'
        pass

    def OnUpdateMediaBegin(self, evt):
        # if LOGWIRE:
        #     log().debug("%s: info=%s", funcname(), self.MediaPlayer.info.dump())
        evt.Enable(self.MediaPlayer.GetState() >= MediaState.Opened)

    def OnMediaEnd(self, evt):
        print 'OnMediaEnd'
        pass

    def OnUpdateMediaEnd(self, evt):
        # if LOGWIRE:
        #     log().debug("%s: info=%s", funcname(), self.MediaPlayer.info.dump())
        evt.Enable(self.MediaPlayer.GetState() >= MediaState.Opened)

    def OnMediaVolumeLow(self, evt):
        self.MediaPlayer.Volume(0)
        self.VolumeSlider.SetValue(0)

    def OnMediaVolumeHigh(self, evt):
        self.MediaPlayer.Volume(100)
        self.VolumeSlider.SetValue(100)

    def OnVolume(self, evt):
        offset = self.VolumeSlider.GetValue()
        self.MediaPlayer.Volume(offset)

    def OnSeek(self, evt):
        timpos = self.MediaSlider.GetValue()
        self.MediaPlayer.Seek(timpos)

    def OnTimer(self, evt):
        offset = self.MediaPlayer.GetCurrTime()
        self.MediaSlider.SetValue(offset)
        # if LOGWIRE:
        #     log().debug("%s: state=%s", funcname(), self.MediaPlayer.ctrl.getState())
        # ct = time.gmtime(offset/1000)
        # print_time = (ct[0], ct[1], ct[2], ct[3], ct[4], ct[5], ct[6], ct[7], -1)
        # self.MediaSlider.SetLabel(time.strftime("%H:%M:%S", ct))
        # self.st_size.SetLabel('size: %s' % self.mc.GetBestSize())
        # self.st_len.SetLabel('length: %d seconds' % (self.mc.Length()/1000))
        # self.st_pos.SetLabel('position: %d' % offset)

