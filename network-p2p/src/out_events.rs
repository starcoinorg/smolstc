// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Registering events streams.
//!
//! This code holds the logic that is used for the network service to inform other parts of
//! Substrate about what is happening.
//!
//! # Usage
//!
//! - Create an instance of [`OutChannels`].
//! - Create channels using the [`channel`] function. The receiving side implements the `Stream`
//! trait.
//! - You cannot directly send an event on a sender. Instead, you have to call
//! [`OutChannels::push`] to put the sender within a [`OutChannels`].
//! - Send events by calling [`OutChannels::send`]. Events are cloned for each sender in the
//! collection.
//!

use crate::Event;

use futures::{channel::mpsc, prelude::*, ready, stream::FusedStream};
use prometheus::Registry;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

/// Creates a new channel that can be associated to a [`OutChannels`].
///
/// The name is used in Prometheus reports.
pub fn channel(name: &'static str) -> (Sender, Receiver) {
    let (tx, rx) = mpsc::unbounded();
    let tx = Sender {
        inner: tx,
        name,
    };
    let rx = Receiver {
        inner: rx,
        name,
    };
    (tx, rx)
}

/// Sending side of a channel.
///
/// Must be associated with an [`OutChannels`] before anything can be sent on it
///
/// > **Note**: Contrary to regular channels, this `Sender` is purposefully designed to not
/// implement the `Clone` trait e.g. in Order to not complicate the logic keeping the metrics in
/// sync on drop. If someone adds a `#[derive(Clone)]` below, it is **wrong**.
pub struct Sender {
    inner: mpsc::UnboundedSender<Event>,
    name: &'static str,
}

impl fmt::Debug for Sender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Sender").finish()
    }
}

/// Receiving side of a channel.
pub struct Receiver {
    inner: mpsc::UnboundedReceiver<Event>,
    name: &'static str,
}

impl Stream for Receiver {
    type Item = Event;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Event>> {
        if let Some(ev) = ready!(Pin::new(&mut self.inner).poll_next(cx)) {
            Poll::Ready(Some(ev))
        } else {
            Poll::Ready(None)
        }
    }
}

impl fmt::Debug for Receiver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Receiver").finish()
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        if !self.inner.is_terminated() {
            // Empty the list to properly decrease the metrics.
            while let Some(Some(_)) = self.next().now_or_never() {}
        }
    }
}

/// Collection of senders.
pub struct OutChannels {
    event_streams: Vec<Sender>,
}

impl OutChannels {
    /// Creates a new empty collection of senders.
    pub fn new(registry: Option<&Registry>) -> Result<Self, anyhow::Error> {
        Ok(OutChannels {
            event_streams: Vec::new(),
        })
    }

    /// Adds a new [`Sender`] to the collection.
    pub fn push(&mut self, sender: Sender) {
        self.event_streams.push(sender);
    }

    /// Sends an event.
    pub fn send(&mut self, event: Event) {
        self.event_streams
            .retain(|sender| sender.inner.unbounded_send(event.clone()).is_ok());
    }
}

impl fmt::Debug for OutChannels {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OutChannels")
            .field("num_channels", &self.event_streams.len())
            .finish()
    }
}
