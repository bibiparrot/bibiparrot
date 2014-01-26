################################################################################
# Name     : MediaPlayControl.py                                               #
# Brief    : Define the base functions to control a rtf file                   #
#                                                                              #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


import os, sys
from ...bibiparrot.Utils.utils import *
import threading
import time

from ..Configurations.configurations import LOGWIRE, log

class MediaState(object):
    __slots__ = []
    Unknown = 0
    Loaded = 1
    Opened = 2
    Playing = 3
    Paused = 4
    Stopped = 5

class MediaType(object):
    __slots__ = []
    Unknown = 0
    Audio = 1
    Video = 2
    CD = 3
    DVD = 4
    Folder = 5


class MediaInfo(Bean):
    __slots__ = ['fpath', 'volume', 'length', 'videosize','starttime', 'currtime','endtime', 'type', 'state']

    def __init__(self, **kwargs):
        Bean.__init__(self, **kwargs)

    def clear(self):
        self.fpath = None
        self.volume = 100
        self.length = 0
        self.starttime = 0
        self.currtime = 0
        self.endtime = 0
        self.videosize = (0, 0)
        self.type = MediaType.Unknown
        self.state = MediaState.Unknown



### @End MusicInfo

class MediaPlayControl(object):
    __slots__ = ['player', 'info']

    def __init__(self, info=None):
        if info is None:
            info = MediaInfo()
        self.info = info
        self.player = None

    ### prepare player ###
    def load(self):
        ### only allowed load once ###
        if self.info.state == MediaState.Unknown:
            if self.load_():
                self.info.state = MediaState.Loaded
        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())


    ### for override load ###
    def load_(self):
        print 'Unimplemented load()'
        return False

    ### prepare the target file ###
    def open(self, curmd=None):
        ### allow always,  if is loaded ###
        if self.info.state >= MediaState.Loaded and os.path.exists(curmd):
            if self.open_(curmd):
                self.info.state = MediaState.Opened
                ### prepare
                self.info.fpath = os.path.abspath(curmd)
                self.info.videosize = self.getVideoSize_()
                self.info.type = self.getMediaType_()
                self.info.length = self.getMediaLength_()

        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())

    def open_(self, curmd):
        print 'Unimplemented open()'
        return False

    # ### start play at certain time position ###
    # def start(self, timpos=None):
    #     if timpos is not None and timpos >=0:
    #         self.timpos = timpos
    #     pass
    ### play the target ###
    def play(self):
        if self.info.state >= MediaState.Opened:
            if self.play_():
                self.info.state = MediaState.Playing
        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())

    def play_(self):
        print 'Unimplemented play()'
        return False

    ### resume paused target ###
    def resume(self):
        if self.info.state == MediaState.Paused:
            if self.resume_():
                self.info.state = MediaState.Playing

        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())

    def resume_(self):
        print 'Unimplemented resume()'
        return False

    ### pause target ###
    def pause(self):
        if self.info.state == MediaState.Playing:
            if self.pause_():
                self.info.state = MediaState.Paused

        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())

    def pause_(self):
        print 'Unimplemented pause()'
        return False

    ### stop play file ###
    def stop(self, fadeout=False, timefadeout=1000):
        if self.info.state >= MediaState.Opened:
            if self.stop_(fadeout, timefadeout):
                self.info.state = MediaState.Stopped
        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())

    def stop_(self, fadeout=False, timefadeout=1000):
        print 'Unimplemented stop()'
        return False

    ### quit the player ###
    def quit(self):
        if self.info.state >= MediaState.Loaded:
            if self.quit_():
                self.info.state = MediaState.Unknown

        if LOGWIRE:
            log().debug("%s: info=%s", funcname(), self.info.dump())

    def quit_(self):
        print 'Unimplemented quit()'
        return False

    ### set the seek (start) position of media ###
    def seek(self, timpos=None):
        if self.info.state >= MediaState.Opened:
            if self.seek_(timpos):
                self.info.currtime = timpos
                return True
        return False


    def seek_(self):
        print 'Unimplemented quit()'
        return False

    ### get the volume ###
    def getVolume(self, default=0):
        if self.info.state >= MediaState.Opened:
            vol = self.getVolume_()
            if vol is not None:
                self.info.volume = vol
                return vol
        return default

    def getVolume_(self):
        print 'Unimplemented getVolume()'
        return None

    ### set the volume ###
    def setVolume(self, volum=None):
        if self.info.state >= MediaState.Opened:
            if self.setVolume_(volum):
                self.info.volume = volum
                return True
        return False

    def setVolume_(self):
        print 'Unimplemented setVolume()'
        return False

    ### return whether is playing ###
    def isPlaying(self):
        if self.info.state == MediaState.Playing:
            if self.isPlaying_():
                return True
            else:
                self.info.state = MediaState.Stopped
        return False

    def isPlaying_(self):
        print 'Unimplemented isPlaying()'
        return False
    ### return the size of video ###
    def getVideoSize(self, default=(0,0)):
        if self.info.state >= MediaState.Opened:
            return self.info.videosize
        return default

    def getVideoSize_(self):
        print 'Unimplemented getVideoSize()'
        return None

    def getMediaType(self, default=MediaType.Unknown):
        if self.info.state >= MediaState.Opened:
            return self.info.type
        return default

    def getMediaType_(self):
        print 'Unimplemented getMediaType()'
        return None

    def getMediaLength(self, default=0):
        if self.info.state >= MediaState.Opened:
            return self.info.length
        return default

    ### return the length of audio ###
    def getMediaLength_(self):
        print 'Unimplemented getMediaLength()'
        return None

    ### return the current play time of audio ###
    def getMediaCurrTime(self, default=0):
        if self.info.state >= MediaState.Opened:
            return self.getMediaCurrTime_()
        return default

    def getMediaCurrTime_(self):
        print 'Unimplemented getMediaCurrTime()'
        return None




### @End MediaPlayControl


###
##  https://wiki.videolan.org/LibVLC/
#

##
#   @Requires: https://wiki.videolan.org/Python_bindings/
#   @URL: vlc.py http://git.videolan.org/?p=vlc/bindings/python.git;a=tree;f=generated;b=HEAD
##

VLC_NEEDED = True
if VLC_NEEDED:
    import vlc

### https://wiki.videolan.org/VLC_Features_Formats/ ###
class VLCMediaPlayCtrl(MediaPlayControl):
    __slots__ = MediaPlayControl.__slots__ + ['instance', 'media', 'handle']
    def __init__(self, handle=None, *args, **kwargs):
        MediaPlayControl.__init__(self, *args, **kwargs)
        self.handle = handle
        self.player = None
        # VLC player controls
        self.instance = vlc.Instance('-I macosx')

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())

    def load_(self):
        self.player = self.instance.media_player_new()
        win_id = int(self.handle)
        if sys.platform == "win32":
            self.player.set_hwnd(win_id)
        elif sys.platform == "darwin":
            # We have to use 'set_nsobject' since Qt4 on OSX uses Cocoa
            # framework and not the old Carbon.
            self.player.set_nsobject(win_id)
        else:
            # for Linux using the X Server
            self.player.set_xwindow(win_id)

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        return True

    def open_(self, curmd=None):
        self.media = self.instance.media_new(unicode(curmd))
        self.player.set_media(self.media)
        self.media.parse()
        print
        print self.player.video_get_track_description()
        print
        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: track=%s, info=%s", funcname(), str(self.player.video_get_track_description()), self.info.dump())
        return True

    # def start(self, timpos=None):
    #     MediaPlayControl.start(self, timpos)
    #     if self.opened and not self.playing:
    #         if self.player.is_seekable():
    #             self.player.set_time(timpos)
    #             self.play()
    #         print 'start'
    #         self.playing= True

    def play_(self):
        if self.player.play() == -1:
            return False

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        return True
        ### http://wxwidgets.10942.n7.nabble.com/Drawing-on-a-wxFrame-via-wxWindow-GetHandle-td12618.html ###

    def pause_(self):
        '''Toggle pause (or resume) media list.
        '''
        if self.player.pause() == -1:
            return False

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        return True

    def resume_(self):

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        return self.pause_()

    def stop_(self, fadeout=False, timefadeout=1000):
        if self.player.stop() == -1:
            return False

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        return True

    def quit_(self):
        if self.player.release() == -1:
            return False

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        return True

    def seek_(self, timpos=None):

        if LOGWIRE:
            log().debug("VLCMediaPlayCtrl %s: info=%s", funcname(), self.info.dump())
        if self.player.is_seekable():
            self.player.set_time(timpos)
            return True
        return False

    ### get the volume ###
    def getVolume_(self):
        return self.player.audio_get_volume()

    ###  vlc.MediaPlayer.audio_set_volume returns 0 if success, -1 otherwise ###
    def setVolume_(self, volum=None):
        self.player.audio_set_volume(volum)


    def isPlayed_(self):
        return self.player.is_playing()

    ### return the size of video ###
    def getVideoSize_(self):
        return self.player.video_get_size()
    ### return the length of audio ###
    def getMediaLength_(self):
        return self.player.get_media().get_duration()
    ### return the current play time of audio ###
    def getMediaCurrTime_(self):
        return self.player.get_time()
