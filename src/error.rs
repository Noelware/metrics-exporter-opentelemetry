// 🐻‍❄️🎈 metrics-exporter-opentelemetry: metrics exporter over OpenTelemetry
// Copyright (c) 2025 Noelware, LLC. <team@noelware.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub type Result<T> = std::result::Result<T, Error>;

/// A error type that occurred.
#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum Error {
    /// *Called from [`Recorder::install_global`]*: Failed to set
    /// a global recorder as one is already initialised.
    ///
    /// [`Recorder::install_global`]: struct.Recorder.html#method.install_global
    SetRecorder(metrics::SetRecorderError<crate::Recorder>),
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::SetRecorder(err) => Some(err),
        }
    }
}
