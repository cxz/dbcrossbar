//! Support for Google Cloud Storage.

use std::{fmt, str::FromStr};

use crate::common::*;
use crate::drivers::bigquery::BigQueryLocator;

mod local_data;
mod prepare_as_destination;
mod write_local_data;
mod write_remote_data;

use local_data::local_data_helper;
pub(crate) use prepare_as_destination::prepare_as_destination_helper;
use write_local_data::write_local_data_helper;
use write_remote_data::write_remote_data_helper;

/// Locator scheme for Google Cloud Storage.
pub(crate) const GS_SCHEME: &str = "gs:";

#[derive(Clone, Debug)]
pub(crate) struct GsLocator {
    url: Url,
}

impl GsLocator {
    /// Access the `gs://` URL in this locator.
    pub(crate) fn as_url(&self) -> &Url {
        &self.url
    }
}

impl fmt::Display for GsLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.url.fmt(f)
    }
}

impl FromStr for GsLocator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with(GS_SCHEME) {
            let url = s
                .parse::<Url>()
                .with_context(|_| format!("cannot parse {}", s))?;
            if !url.path().starts_with('/') {
                Err(format_err!("{} must start with gs://", url))
            } else if !url.path().ends_with('/') {
                Err(format_err!("{} must end with a '/'", url))
            } else {
                Ok(GsLocator { url })
            }
        } else {
            Err(format_err!("expected {} to begin with gs://", s))
        }
    }
}

impl Locator for GsLocator {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn local_data(
        &self,
        ctx: Context,
        shared_args: SharedArguments<Unverified>,
        source_args: SourceArguments<Unverified>,
    ) -> BoxFuture<Option<BoxStream<CsvStream>>> {
        local_data_helper(ctx, self.url.clone(), shared_args, source_args).boxed()
    }

    fn write_local_data(
        &self,
        ctx: Context,
        data: BoxStream<CsvStream>,
        shared_args: SharedArguments<Unverified>,
        dest_args: DestinationArguments<Unverified>,
    ) -> BoxFuture<BoxStream<BoxFuture<()>>> {
        write_local_data_helper(ctx, self.url.clone(), data, shared_args, dest_args)
            .boxed()
    }

    fn supports_write_remote_data(&self, source: &dyn Locator) -> bool {
        // We can only do `write_remote_data` if `source` is a `BigQueryLocator`.
        // Otherwise, we need to do `write_local_data` like normal.
        source.as_any().is::<BigQueryLocator>()
    }

    fn write_remote_data(
        &self,
        ctx: Context,
        source: BoxLocator,
        shared_args: SharedArguments<Unverified>,
        source_args: SourceArguments<Unverified>,
        dest_args: DestinationArguments<Unverified>,
    ) -> BoxFuture<()> {
        write_remote_data_helper(
            ctx,
            source,
            self.to_owned(),
            shared_args,
            source_args,
            dest_args,
        )
        .boxed()
    }
}

impl LocatorStatic for GsLocator {
    fn features() -> Features {
        Features {
            locator: LocatorFeatures::LOCAL_DATA | LocatorFeatures::WRITE_LOCAL_DATA,
            write_schema_if_exists: IfExistsFeatures::empty(),
            source_args: SourceArgumentsFeatures::empty(),
            dest_args: DestinationArgumentsFeatures::empty(),
            dest_if_exists: IfExistsFeatures::OVERWRITE,
            _placeholder: (),
        }
    }
}
