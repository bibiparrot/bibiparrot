#-------------------------------------------------------------------------------
# Name:        Audio Control Module for bibiParrot
# Purpose:     This class provide functions to support the media player in bibiparrot
#
# Author:      S Xia
# Version:     0.3a
#
# Created:     01/09/2014
# Copyright:   (c) S Xia 2014
# Licence:     This tool is licensed for bibiparrot only.
#-------------------------------------------------------------------------------

#-------------------------------------------------------------------------------
# Error Code:
# 1 - File cannot be found
# 2 - Cannot load music
#-------------------------------------------------------------------------------

### Import Statement
import pygame
import os
import threading
import time
####################

## This fuunction init the media player, please call this function first.
def initMediaPlayer():
    pygame.init()
    pygame.mixer.init()
    pass

## Call this function to load music
def loadMusic(myMusicPath):
    if not os.path.exists(myMusicPath):
        print "File cannot be found! "
        exit(1)
    fp = open(myMusicPath)
    try:
        pygame.mixer.music.load(fp)
    except:
        print "Unexpected error happened! "
        exit(2)
    pass

## Start playing music
def playMusicSingle():
    pygame.mixer.music.play()
    pass

## Pause the music
def playMusicPause():
    pygame.mixer.music.pause()
    pass

## Stop the music
def playMusicStop():
    pygame.mixer.music.stop()
    pass

## Music Control -- Inner Use Only
def playMusicCtrl(loop, startTime, endTime):
    pass

## Call this function when finished
def destroyPlayer():
    pygame.mixer.quit()
    pass

### Define the main class (Test Purposes)
def main():
    print " *** Audio Utils Test Program *** "
    import inspect
    mp3dir = os.path.join(os.path.dirname(inspect.getfile(inspect.currentframe())), 'testMusics')
    myMusicPath = os.path.join(mp3dir, r'Beijing Welcome You.mp3')
    print 'MP3 file is', myMusicPath
    print "Init the music player.. "
    initMediaPlayer()
    print "Load current music.. "
    loadMusic(myMusicPath)
    print "Start playing music without repeat/stop.. "
    playMusicSingle()

    pass
#########################

### Define the Universal Controller
if __name__ == '__main__':
    main()
###################################