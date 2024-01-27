/* Communication module for Android terminals.  -*- c-file-style: "GNU" -*-

Copyright (C) 2023-2024 Free Software Foundation, Inc.

This file is part of GNU Emacs.

GNU Emacs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or (at
your option) any later version.

GNU Emacs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with GNU Emacs.  If not, see <https://www.gnu.org/licenses/>.  */

package org.gnu.emacs;

import android.app.Notification;
import android.app.NotificationManager;
import android.app.NotificationChannel;
import android.app.PendingIntent;

import android.content.Context;
import android.content.Intent;

import android.os.Build;

import android.widget.RemoteViews;



/* Structure designating a single desktop notification.

   New versions of Android also organize notifications into individual
   ``channels'', which are used to implement groups.  Unlike on other
   systems, notification importance is set for each group, not for
   each individual notification.  */



public final class EmacsDesktopNotification
{
  /* The content of this desktop notification.  */
  public final String content;

  /* The title of this desktop notification.  */
  public final String title;

  /* The notification group.  */
  public final String group;

  /* String identifying this notification for future replacement.
     Typically a string resembling ``XXXX.NNNN.YYYY'', where XXXX is
     the system boot time, NNNN is the PID of this Emacs instance, and
     YYYY is the counter value returned by the notifications display
     function.  */
  public final String tag;

  /* The identifier of this notification's icon.  */
  public final int icon;

  /* The importance of this notification's group.  */
  public final int importance;

  public
  EmacsDesktopNotification (String title, String content,
			    String group, String tag, int icon,
			    int importance)
  {
    this.content    = content;
    this.title	    = title;
    this.group	    = group;
    this.tag        = tag;
    this.icon       = icon;
    this.importance = importance;
  }



  /* Functions for displaying desktop notifications.  */

  /* Internal helper for `display' executed on the main thread.  */

  @SuppressWarnings ("deprecation") /* Notification.Builder (Context).  */
  private void
  display1 (Context context)
  {
    NotificationManager manager;
    NotificationChannel channel;
    Notification notification;
    Object tem;
    RemoteViews contentView;
    Intent intent;
    PendingIntent pending;
    int priority;

    tem = context.getSystemService (Context.NOTIFICATION_SERVICE);
    manager = (NotificationManager) tem;

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O)
      {
	/* Create the notification channel for this group.  If a group
	   already exists with the same name, its linked attributes
	   (such as its importance) will be overridden.  */
        channel = new NotificationChannel (group, group, importance);
	manager.createNotificationChannel (channel);

	/* Create a notification object and display it.  */
	notification = (new Notification.Builder (context, group)
			.setContentTitle (title)
			.setContentText (content)
			.setSmallIcon (icon)
			.build ());
      }
    else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.HONEYCOMB)
      {
	/* Android 7.1 and earlier don't segregate notifications into
	   distinct categories, but permit an importance to be
	   assigned to each individual notification.  */

	switch (importance)
	  {
	  case 2: /* IMPORTANCE_LOW */
	  default:
	    priority = Notification.PRIORITY_LOW;
	    break;

	  case 3: /* IMPORTANCE_DEFAULT */
	    priority = Notification.PRIORITY_DEFAULT;
	    break;

	  case 4: /* IMPORTANCE_HIGH */
	    priority = Notification.PRIORITY_HIGH;
	    break;
	  }

	notification = (new Notification.Builder (context)
			.setContentTitle (title)
			.setContentText (content)
			.setSmallIcon (icon)
			.setPriority (priority)
			.build ());

	if (Build.VERSION.SDK_INT > Build.VERSION_CODES.JELLY_BEAN)
	  notification.priority = priority;
      }
    else
      {
	notification = new Notification ();
	notification.icon = icon;

	/* This remote widget tree is defined in
	   java/res/layout/sdk8_notifications_view.xml.  */
	notification.contentView
	  = contentView
	  = new RemoteViews ("org.gnu.emacs",
			     R.layout.sdk8_notifications_view);
	contentView.setTextViewText (R.id.sdk8_notifications_title,
				     title);
	contentView.setTextViewText (R.id.sdk8_notifications_content,
				     content);
      }

    /* Provide a content intent which starts Emacs when the
       notification is clicked.  */

    intent = new Intent (context, EmacsActivity.class);
    intent.addFlags (Intent.FLAG_ACTIVITY_NEW_TASK);

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)
      pending = PendingIntent.getActivity (context, 0, intent,
					   PendingIntent.FLAG_IMMUTABLE);
    else
      pending = PendingIntent.getActivity (context, 0, intent, 0);

    notification.contentIntent = pending;

    manager.notify (tag, 2, notification);
  }

  /* Display this desktop notification.

     Create a notification channel named GROUP or update its
     importance if such a channel is already defined.  */

  public void
  display ()
  {
    EmacsService.SERVICE.runOnUiThread (new Runnable () {
	@Override
	public void
	run ()
	{
	  display1 (EmacsService.SERVICE);
	}
      });
  }
};
