################################################################################
# Name     : MediaPlayControl.py                                                  #
# Brief    : Define the base functions to control a rtf file                   #
#                                                                              #
# Url      :                                                                   #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


import os, sys
from ...bibiparrot.Utils.utils import Bean
import threading
import time


class MediaInfo(Bean):
    ##  ##
    __slots__ = ['fpath', 'timlen']

### @End MusicInfo

class MediaPlayControl(Bean):
    __slots__ = ['player','mdlists', 'curmd','bufsize','volum', 'timpos',
                'loaded', 'opened', 'played', 'paused']
    def __init__(self, *args, **kwargs):
        Bean.__init__(self,**kwargs)
        self.mdlists = []
        self.volum = 0
        self.timpos = 0
        self.loaded = False
        self.opened = False
        self.played = False
        self.paused = False

    ### prepare player ###
    def load(self):
        pass
    ### prepare the target file ###
    def open(self, curmd=None):
        if curmd is not None:
            self.curmd = curmd
        pass
    ### start play at certain time position ###
    def start(self, timpos=None):
        if timpos is not None and timpos >=0:
            self.timpos = timpos
        pass
    ### play the target ###
    def play(self):
        pass
    ### resume paused target ###
    def resume(self):
        pass
    ### pause target ###
    def pause(self):
        pass
    ### stop play file ###
    def stop(self, fadeout=False, timefadeout=1000):
        pass
    ### quit the player ###
    def quit(self):
        pass
    ### set the volume ###
    def setVolume(self, volum=None):
        if volum:
            self.volum = volum
    ### set the position of media ###
    def setPosition(self, timpos=None):
        if timpos:
            self.timpos = timpos
    ### return whether is playing ###
    def isPlayed(self):
        return self.played



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

class VLCMediaPlayCtrl(MediaPlayControl):
    __slots__ = MediaPlayControl.__slots__ + ['instance', 'media', 'panel']
    def __init__(self,handle=None, *args, **kwargs):
        MediaPlayControl.__init__(self, *args, **kwargs)
        self.handle = handle
        self.player = None
        # VLC player controls
        self.instance = vlc.Instance('-I macosx')
        pass

    def load(self):
        MediaPlayControl.load(self)
        if not self.loaded:
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
            print 'load'
            self.loaded = True
            self.opened = False
            self.played = False
            self.paused = False

        pass

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.load ###
    def open(self, curmd=None):
        MediaPlayControl.open(self, curmd)
        if self.loaded and os.path.exists(self.curmd) and not self.opened:
            self.media = self.instance.media_new(unicode(curmd))
            self.player.set_media(self.media)
            self.media.parse()
            print 'open'
            self.opened = True

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.play ###
    def start(self, timpos=None):
        MediaPlayControl.start(self, timpos)
        if self.opened and not self.played:
            if self.player.is_seekable():
                self.player.set_time(timpos)
                self.play()
            print 'start'
            self.played = True

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.play ###
    def play(self):
        MediaPlayControl.play(self)
        if self.player.play() == -1:
            print ("Unable to play.")
        self.played = True
        ### http://wxwidgets.10942.n7.nabble.com/Drawing-on-a-wxFrame-via-wxWindow-GetHandle-td12618.html ###


    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.pause ###
    def pause(self):
        MediaPlayControl.pause(self)
        if self.played and not self.paused:
            self.player.pause()
            self.paused = True

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.unpause ###
    def resume(self):
        MediaPlayControl.resume(self)
        if self.played and self.paused:
            self.player.unpause()
            self.paused = False

    def stop(self, fadeout=False, timefadeout=1000):
        MediaPlayControl.stop(self, fadeout)
        if self.played:
            self.player.stop()
            self.played = False
            print 'stop'

    ### http://www.pygame.org/docs/ref/mixer.html#pygame.mixer.quit ###
    def quit(self):
        MediaPlayControl.quit(self)
        if self.loaded:
            self.mixer.quit()
        self.played = False
        self.paused = False
        self.opened = False
        self.loaded = False
        print 'quit'

    ###  vlc.MediaPlayer.audio_set_volume returns 0 if success, -1 otherwise ###
    def setVolume(self, volum=None):
        MediaPlayControl.setVolume(self,volum)
        if self.loaded:
            if self.player.audio_set_volume(volum) == -1:
                print ("Failed to set volume")

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.set_pos ###
    def setPosition(self, timpos=None):
        MediaPlayControl.setPosition(self,timpos)

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.get_busy ###
    def isPlayed(self):
        if not MediaPlayControl.isPlayed(self):
            return False
        self.played = self.player.get_busy()
        return self.played





###
##  http://www.pygame.org/docs/ref/mixer.html
#

###
# unfortunately, pygame do not support Mac OX S (at least 10.9) well. It crashes when reaping start(10).
###
PYGAME_NEEDED = True

if PYGAME_NEEDED:
    import pygame

class PygameMediaPlayCtrl(MediaPlayControl):
    ### http://stackoverflow.com/questions/1720421/merge-two-lists-in-python ###
    __slots__ = MediaPlayControl.__slots__ + ['mixer']
    def __init__(self, bufsize=4096*2, *args, **kwargs):
        MediaPlayControl.__init__(self, *args, bufsize=bufsize, **kwargs)
        ### http://www.pygame.org/docs/ref/mixer.html ###
        self.mixer = pygame.mixer
        self.player = self.mixer.music
        ### Must larger than default size of pygame ###
        pass

    def load(self):
        MediaPlayControl.load(self)
        if not self.loaded:
            ### Some platforms require the pygame module for loading and playing sounds module to   ###
            ###   be initialized after the display modules have initialized                         ###
            if (self.bufsize < 4096):
                self.bufsize = 4096
            self.mixer.pre_init(buffer=self.bufsize)
            self.mixer.init()
            print 'load'
            self.loaded = True
            self.opened = False
            self.played = False
            self.paused = False

        pass

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.load ###
    def open(self, curmd=None):
        MediaPlayControl.open(self, curmd)
        if self.loaded and os.path.exists(self.curmd) and not self.opened:
            self.player.load(self.curmd)
            print 'open'
            self.opened = True

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.play ###
    def start(self, timpos=None):
        MediaPlayControl.start(self, timpos)
        if self.opened and not self.played:
            self.player.play(0,timpos)
            print 'start'
            self.played = True

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.play ###
    def play(self):
        MediaPlayControl.play(self)
        self.start(self.timpos)

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.pause ###
    def pause(self):
        MediaPlayControl.pause(self)
        if self.played and not self.paused:
            self.player.pause()
            self.paused = True

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.unpause ###
    def resume(self):
        MediaPlayControl.resume(self)
        if self.played and self.paused:
            self.player.unpause()
            self.paused = False

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.stop ###
    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.fadeout ###
    def stop(self, fadeout=False, timefadeout=1000):
        MediaPlayControl.stop(self, fadeout)
        if self.played:
            self.player.unpause()
            if fadeout:
                self.player.fadeout(timefadeout)
            else:
                self.player.stop()
            self.played = False
            print 'stop'

    ### http://www.pygame.org/docs/ref/mixer.html#pygame.mixer.quit ###
    def quit(self):
        MediaPlayControl.quit(self)
        if self.loaded:
            self.mixer.quit()
        self.played = False
        self.paused = False
        self.opened = False
        self.loaded = False
        print 'quit'

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.set_volume ###
    def setVolume(self, volum=None):
        MediaPlayControl.setVolume(self,volum)
        if self.loaded:
            self.player.set_volume(self.volum)

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.set_pos ###
    def setPosition(self, timpos=None):
        MediaPlayControl.setPosition(self,timpos)

    ### http://www.pygame.org/docs/ref/music.html#pygame.mixer.music.get_busy ###
    def isPlayed(self):
        if not MediaPlayControl.isPlayed(self):
            return False
        self.played = self.player.get_busy()
        return self.played

### @End PygameMediaPlayCtrl