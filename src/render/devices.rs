use std::fmt;

use anyhow::{Context, Result, bail};
use comrak::adapters::CodefenceRendererAdapter;
use comrak::html::{escape, escape_href};
use comrak::nodes::Sourcepos;
use serde::Deserialize;

use super::DeviceCards;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Device {
    name: String,
    description: Option<String>,
    image: Option<String>,
    #[serde(default)]
    specs: Vec<String>,
}

impl CodefenceRendererAdapter for DeviceCards {
    fn write(
        &self,
        output: &mut dyn fmt::Write,
        _language: &str,
        _metadata: &str,
        code: &str,
        _source_position: Option<Sourcepos>,
    ) -> fmt::Result {
        let devices = match parse(code) {
            Ok(devices) => devices,
            Err(error) => {
                *self.error.lock().unwrap() = Some(error);

                return Err(fmt::Error);
            }
        };

        render(output, &devices)
    }
}

fn parse(source: &str) -> Result<Vec<Device>> {
    let mut devices: Vec<Device> = yaml_serde::from_str(source).context("invalid devices block")?;
    if devices.is_empty() {
        bail!("devices block cannot be empty");
    }

    for (index, device) in devices.iter_mut().enumerate() {
        device.name = device.name.trim().to_owned();
        device.description = device
            .description
            .take()
            .map(|description| description.trim().to_owned())
            .filter(|description| !description.is_empty());
        device.image = device
            .image
            .take()
            .map(|image| image.trim().to_owned())
            .filter(|image| !image.is_empty());
        device.specs = std::mem::take(&mut device.specs)
            .into_iter()
            .map(|spec| spec.trim().to_owned())
            .filter(|spec| !spec.is_empty())
            .collect();

        if device.name.is_empty() {
            bail!("devices entry {} has an empty `name`", index + 1);
        }
    }

    Ok(devices)
}

fn render(output: &mut dyn fmt::Write, devices: &[Device]) -> fmt::Result {
    output.write_str("<ul class=\"device-list\">\n")?;
    for device in devices {
        output.write_str("<li class=\"device-card\">\n")?;
        if let Some(image) = device.image.as_deref() {
            output.write_str("<div class=\"device-card-image\"><img src=\"")?;
            escape_href(output, image, false)?;
            output.write_str("\" alt=\"")?;
            escape(output, &device.name)?;
            output.write_str("\" loading=\"lazy\" decoding=\"async\"></div>\n")?;
        }
        output.write_str("<div class=\"device-card-content\">\n<h3 class=\"device-card-name\">")?;
        escape(output, &device.name)?;
        output.write_str("</h3>\n")?;
        if let Some(description) = device.description.as_deref() {
            output.write_str("<p class=\"device-card-description\">")?;
            escape(output, description)?;
            output.write_str("</p>\n")?;
        }
        if !device.specs.is_empty() {
            output.write_str("<ul class=\"device-specs\">\n")?;
            for spec in &device.specs {
                output.write_str("<li>")?;
                escape(output, spec)?;
                output.write_str("</li>\n")?;
            }
            output.write_str("</ul>\n")?;
        }
        output.write_str("</div>\n</li>\n")?;
    }

    output.write_str("</ul>\n")
}
