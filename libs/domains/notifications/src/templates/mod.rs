//! Email template rendering engine.
//!
//! This module provides Handlebars-based template rendering for emails.

use crate::error::{NotificationError, NotificationResult};
use crate::models::{
    EmailType, PasswordResetEmailData, TaskNotificationData, WeeklyDigestData, WelcomeEmailData,
};
use handlebars::Handlebars;
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

/// Rendered email content.
#[derive(Debug, Clone)]
pub struct RenderedEmail {
    /// HTML body content.
    pub html: String,
    /// Plain text body content.
    pub text: String,
    /// Email subject line.
    pub subject: String,
}

/// Template engine for rendering email templates.
pub struct TemplateEngine {
    handlebars: Arc<Handlebars<'static>>,
}

impl TemplateEngine {
    /// Create a new template engine with all templates registered.
    pub fn new() -> NotificationResult<Self> {
        let mut handlebars = Handlebars::new();

        // Register welcome email templates
        handlebars
            .register_template_string("welcome_html", WELCOME_HTML_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register welcome_html: {}", e)))?;
        handlebars
            .register_template_string("welcome_text", WELCOME_TEXT_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register welcome_text: {}", e)))?;

        // Register verification email templates
        handlebars
            .register_template_string("verification_html", VERIFICATION_HTML_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register verification_html: {}", e)))?;
        handlebars
            .register_template_string("verification_text", VERIFICATION_TEXT_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register verification_text: {}", e)))?;

        // Register password reset email templates
        handlebars
            .register_template_string("password_reset_html", PASSWORD_RESET_HTML_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register password_reset_html: {}", e)))?;
        handlebars
            .register_template_string("password_reset_text", PASSWORD_RESET_TEXT_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register password_reset_text: {}", e)))?;

        // Register task notification templates
        handlebars
            .register_template_string("task_notification_html", TASK_NOTIFICATION_HTML_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register task_notification_html: {}", e)))?;
        handlebars
            .register_template_string("task_notification_text", TASK_NOTIFICATION_TEXT_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register task_notification_text: {}", e)))?;

        // Register weekly digest templates
        handlebars
            .register_template_string("weekly_digest_html", WEEKLY_DIGEST_HTML_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register weekly_digest_html: {}", e)))?;
        handlebars
            .register_template_string("weekly_digest_text", WEEKLY_DIGEST_TEXT_TEMPLATE)
            .map_err(|e| NotificationError::TemplateError(format!("Failed to register weekly_digest_text: {}", e)))?;

        Ok(Self {
            handlebars: Arc::new(handlebars),
        })
    }

    /// Render a template with the given data.
    fn render<T: Serialize>(&self, template_name: &str, data: &T) -> NotificationResult<String> {
        self.handlebars
            .render(template_name, data)
            .map_err(|e| NotificationError::TemplateError(e.to_string()))
    }

    /// Render a welcome email.
    pub fn render_welcome(&self, data: &WelcomeEmailData) -> NotificationResult<RenderedEmail> {
        debug!(user = %data.user_name, "Rendering welcome email");

        let html = self.render("welcome_html", data)?;
        let text = self.render("welcome_text", data)?;

        Ok(RenderedEmail {
            html,
            text,
            subject: format!("Welcome to Zerg, {}!", data.user_name),
        })
    }

    /// Render an email verification email (standalone, not combined with welcome).
    pub fn render_verification(&self, data: &WelcomeEmailData) -> NotificationResult<RenderedEmail> {
        debug!(user = %data.user_name, "Rendering verification email");

        let html = self.render("verification_html", data)?;
        let text = self.render("verification_text", data)?;

        Ok(RenderedEmail {
            html,
            text,
            subject: "Verify your email address".to_string(),
        })
    }

    /// Render a password reset email.
    pub fn render_password_reset(&self, data: &PasswordResetEmailData) -> NotificationResult<RenderedEmail> {
        debug!(user = %data.user_name, "Rendering password reset email");

        let html = self.render("password_reset_html", data)?;
        let text = self.render("password_reset_text", data)?;

        Ok(RenderedEmail {
            html,
            text,
            subject: "Reset your password".to_string(),
        })
    }

    /// Render a task notification email.
    pub fn render_task_notification(&self, data: &TaskNotificationData) -> NotificationResult<RenderedEmail> {
        debug!(user = %data.user_name, notification_type = %data.notification_type, "Rendering task notification email");

        let html = self.render("task_notification_html", data)?;
        let text = self.render("task_notification_text", data)?;

        let subject = match data.notification_type.as_str() {
            "assigned" => format!("Task assigned: {}", data.task_title),
            "due_soon" => format!("Task due soon: {}", data.task_title),
            "overdue" => format!("Task overdue: {}", data.task_title),
            "completed" => format!("Task completed: {}", data.task_title),
            _ => format!("Task update: {}", data.task_title),
        };

        Ok(RenderedEmail {
            html,
            text,
            subject,
        })
    }

    /// Render a weekly digest email.
    pub fn render_weekly_digest(&self, data: &WeeklyDigestData) -> NotificationResult<RenderedEmail> {
        debug!(user = %data.user_name, "Rendering weekly digest email");

        let html = self.render("weekly_digest_html", data)?;
        let text = self.render("weekly_digest_text", data)?;

        Ok(RenderedEmail {
            html,
            text,
            subject: "Your weekly productivity digest".to_string(),
        })
    }

    /// Render an email by type using JSON data.
    pub fn render_by_type(
        &self,
        email_type: &EmailType,
        data: &serde_json::Value,
    ) -> NotificationResult<RenderedEmail> {
        match email_type {
            EmailType::Welcome => {
                let typed_data: WelcomeEmailData = serde_json::from_value(data.clone())?;
                self.render_welcome(&typed_data)
            }
            EmailType::Verification => {
                let typed_data: WelcomeEmailData = serde_json::from_value(data.clone())?;
                self.render_verification(&typed_data)
            }
            EmailType::PasswordReset => {
                let typed_data: PasswordResetEmailData = serde_json::from_value(data.clone())?;
                self.render_password_reset(&typed_data)
            }
            EmailType::TaskNotification => {
                let typed_data: TaskNotificationData = serde_json::from_value(data.clone())?;
                self.render_task_notification(&typed_data)
            }
            EmailType::WeeklyDigest => {
                let typed_data: WeeklyDigestData = serde_json::from_value(data.clone())?;
                self.render_weekly_digest(&typed_data)
            }
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default template engine")
    }
}

// ============================================================================
// Email Templates
// ============================================================================

const WELCOME_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Welcome to Zerg</title>
</head>
<body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #f4f4f5;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; padding: 40px 20px;">
    <tr>
      <td style="background-color: #ffffff; border-radius: 8px; padding: 40px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center; padding-bottom: 30px;">
              <img src="{{logo_url}}" alt="Zerg" height="40" style="display: inline-block;">
            </td>
          </tr>
        </table>
        <h1 style="color: #18181b; font-size: 24px; font-weight: 600; margin: 0 0 16px 0; text-align: center;">
          Welcome to Zerg, {{user_name}}!
        </h1>
        <p style="color: #52525b; font-size: 16px; line-height: 24px; margin: 0 0 24px 0; text-align: center;">
          Thanks for joining us. Your account has been created successfully and you're ready to start managing your tasks and projects.
        </p>
        {{#if requires_verification}}
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-bottom: 32px;">
          <tr>
            <td style="background-color: #fef3c7; border-radius: 8px; padding: 16px; border-left: 4px solid #f59e0b;">
              <p style="color: #92400e; font-size: 14px; margin: 0;">
                <strong>Action Required:</strong> Please verify your email address to unlock all features.
              </p>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center;">
              <a href="{{verification_url}}" style="display: inline-block; background-color: #2563eb; color: #ffffff; font-size: 16px; font-weight: 500; padding: 12px 32px; text-decoration: none; border-radius: 6px;">
                Verify Email Address
              </a>
            </td>
          </tr>
        </table>
        <p style="color: #71717a; font-size: 12px; text-align: center; margin: 16px 0 32px 0;">
          This link expires in {{verification_expiry_hours}} hours.
        </p>
        {{/if}}
        <table width="100%" cellspacing="0" cellpadding="0" style="border-top: 1px solid #e4e4e7; padding-top: 32px;">
          <tr>
            <td>
              <h2 style="color: #18181b; font-size: 18px; font-weight: 600; margin: 0 0 20px 0;">
                Get Started in 3 Steps
              </h2>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-bottom: 16px;">
          <tr>
            <td width="40" valign="top">
              <div style="width: 32px; height: 32px; background-color: #dbeafe; border-radius: 50%; text-align: center; line-height: 32px; color: #2563eb; font-weight: 600;">1</div>
            </td>
            <td style="padding-left: 12px;">
              <p style="color: #18181b; font-size: 14px; font-weight: 500; margin: 0 0 4px 0;">Create Your First Project</p>
              <p style="color: #71717a; font-size: 14px; margin: 0;">Organize your work into projects for better management.</p>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-bottom: 16px;">
          <tr>
            <td width="40" valign="top">
              <div style="width: 32px; height: 32px; background-color: #dbeafe; border-radius: 50%; text-align: center; line-height: 32px; color: #2563eb; font-weight: 600;">2</div>
            </td>
            <td style="padding-left: 12px;">
              <p style="color: #18181b; font-size: 14px; font-weight: 500; margin: 0 0 4px 0;">Add Tasks</p>
              <p style="color: #71717a; font-size: 14px; margin: 0;">Break down your work into actionable tasks with deadlines.</p>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-bottom: 32px;">
          <tr>
            <td width="40" valign="top">
              <div style="width: 32px; height: 32px; background-color: #dbeafe; border-radius: 50%; text-align: center; line-height: 32px; color: #2563eb; font-weight: 600;">3</div>
            </td>
            <td style="padding-left: 12px;">
              <p style="color: #18181b; font-size: 14px; font-weight: 500; margin: 0 0 4px 0;">Track Progress</p>
              <p style="color: #71717a; font-size: 14px; margin: 0;">Monitor your productivity and celebrate completions.</p>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center;">
              <a href="{{dashboard_url}}" style="display: inline-block; background-color: #18181b; color: #ffffff; font-size: 16px; font-weight: 500; padding: 12px 32px; text-decoration: none; border-radius: 6px;">
                Go to Dashboard
              </a>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-top: 32px; border-top: 1px solid #e4e4e7; padding-top: 24px;">
          <tr>
            <td>
              <h3 style="color: #71717a; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; margin: 0 0 12px 0;">
                Your Account Details
              </h3>
            </td>
          </tr>
          <tr>
            <td style="background-color: #f4f4f5; border-radius: 6px; padding: 16px;">
              <p style="color: #52525b; font-size: 14px; margin: 0 0 8px 0;">
                <strong>Email:</strong> {{user_email}}
              </p>
              <p style="color: #52525b; font-size: 14px; margin: 0 0 8px 0;">
                <strong>Account Type:</strong> {{account_type}}
              </p>
              <p style="color: #52525b; font-size: 14px; margin: 0;">
                <strong>Joined:</strong> {{join_date}}
              </p>
            </td>
          </tr>
        </table>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-top: 24px;">
          <tr>
            <td style="background-color: #f0fdf4; border-radius: 6px; padding: 16px; border-left: 4px solid #22c55e;">
              <p style="color: #166534; font-size: 13px; margin: 0;">
                <strong>Security Tip:</strong> Enable two-factor authentication in your account settings for added protection.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
    <tr>
      <td style="padding: 24px 0; text-align: center;">
        <p style="color: #71717a; font-size: 12px; margin: 0 0 8px 0;">
          You're receiving this email because you signed up for Zerg.
        </p>
        <p style="color: #71717a; font-size: 12px; margin: 0 0 16px 0;">
          <a href="{{preferences_url}}" style="color: #2563eb; text-decoration: none;">Email Preferences</a> |
          <a href="{{help_url}}" style="color: #2563eb; text-decoration: none;">Help Center</a> |
          <a href="{{privacy_url}}" style="color: #2563eb; text-decoration: none;">Privacy Policy</a>
        </p>
        <p style="color: #a1a1aa; font-size: 11px; margin: 0;">
          {{company_name}} | {{company_address}}
        </p>
      </td>
    </tr>
  </table>
</body>
</html>"#;

const WELCOME_TEXT_TEMPLATE: &str = r#"Welcome to Zerg, {{user_name}}!

Thanks for joining us. Your account has been created successfully.

{{#if requires_verification}}
ACTION REQUIRED: Please verify your email address
Click here: {{verification_url}}
(Link expires in {{verification_expiry_hours}} hours)
{{/if}}

GET STARTED IN 3 STEPS
======================
1. Create Your First Project - Organize your work into projects
2. Add Tasks - Break down work into actionable items
3. Track Progress - Monitor productivity and celebrate wins

Go to your dashboard: {{dashboard_url}}

YOUR ACCOUNT DETAILS
====================
Email: {{user_email}}
Account Type: {{account_type}}
Joined: {{join_date}}

SECURITY TIP: Enable two-factor authentication for added protection.

---
Email Preferences: {{preferences_url}}
Help Center: {{help_url}}
Privacy Policy: {{privacy_url}}

{{company_name}} | {{company_address}}"#;

const VERIFICATION_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Verify Your Email</title>
</head>
<body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #f4f4f5;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; padding: 40px 20px;">
    <tr>
      <td style="background-color: #ffffff; border-radius: 8px; padding: 40px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center; padding-bottom: 30px;">
              <img src="{{logo_url}}" alt="Zerg" height="40" style="display: inline-block;">
            </td>
          </tr>
        </table>
        <h1 style="color: #18181b; font-size: 24px; font-weight: 600; margin: 0 0 16px 0; text-align: center;">
          Verify your email address
        </h1>
        <p style="color: #52525b; font-size: 16px; line-height: 24px; margin: 0 0 24px 0; text-align: center;">
          Hi {{user_name}}, please click the button below to verify your email address and activate your account.
        </p>
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center;">
              <a href="{{verification_url}}" style="display: inline-block; background-color: #2563eb; color: #ffffff; font-size: 16px; font-weight: 500; padding: 12px 32px; text-decoration: none; border-radius: 6px;">
                Verify Email Address
              </a>
            </td>
          </tr>
        </table>
        <p style="color: #71717a; font-size: 12px; text-align: center; margin: 16px 0 0 0;">
          This link expires in {{verification_expiry_hours}} hours.
        </p>
        <p style="color: #71717a; font-size: 12px; text-align: center; margin: 24px 0 0 0;">
          If you didn't create an account, you can safely ignore this email.
        </p>
      </td>
    </tr>
    <tr>
      <td style="padding: 24px 0; text-align: center;">
        <p style="color: #a1a1aa; font-size: 11px; margin: 0;">
          {{company_name}} | {{company_address}}
        </p>
      </td>
    </tr>
  </table>
</body>
</html>"#;

const VERIFICATION_TEXT_TEMPLATE: &str = r#"Verify your email address

Hi {{user_name}},

Please click the link below to verify your email address and activate your account:

{{verification_url}}

This link expires in {{verification_expiry_hours}} hours.

If you didn't create an account, you can safely ignore this email.

---
{{company_name}} | {{company_address}}"#;

const PASSWORD_RESET_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Reset Your Password</title>
</head>
<body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #f4f4f5;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; padding: 40px 20px;">
    <tr>
      <td style="background-color: #ffffff; border-radius: 8px; padding: 40px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center; padding-bottom: 30px;">
              <img src="{{logo_url}}" alt="Zerg" height="40" style="display: inline-block;">
            </td>
          </tr>
        </table>
        <h1 style="color: #18181b; font-size: 24px; font-weight: 600; margin: 0 0 16px 0; text-align: center;">
          Reset your password
        </h1>
        <p style="color: #52525b; font-size: 16px; line-height: 24px; margin: 0 0 24px 0; text-align: center;">
          Hi {{user_name}}, we received a request to reset your password. Click the button below to create a new password.
        </p>
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center;">
              <a href="{{reset_url}}" style="display: inline-block; background-color: #dc2626; color: #ffffff; font-size: 16px; font-weight: 500; padding: 12px 32px; text-decoration: none; border-radius: 6px;">
                Reset Password
              </a>
            </td>
          </tr>
        </table>
        <p style="color: #71717a; font-size: 12px; text-align: center; margin: 16px 0 0 0;">
          This link expires in {{expiry_hours}} hour(s).
        </p>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-top: 24px;">
          <tr>
            <td style="background-color: #fef2f2; border-radius: 6px; padding: 16px; border-left: 4px solid #dc2626;">
              <p style="color: #991b1b; font-size: 13px; margin: 0;">
                <strong>Security Notice:</strong> If you didn't request this password reset, please ignore this email. Your password will remain unchanged.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
    <tr>
      <td style="padding: 24px 0; text-align: center;">
        <p style="color: #71717a; font-size: 12px; margin: 0 0 16px 0;">
          <a href="{{help_url}}" style="color: #2563eb; text-decoration: none;">Need help?</a>
        </p>
        <p style="color: #a1a1aa; font-size: 11px; margin: 0;">
          {{company_name}} | {{company_address}}
        </p>
      </td>
    </tr>
  </table>
</body>
</html>"#;

const PASSWORD_RESET_TEXT_TEMPLATE: &str = r#"Reset your password

Hi {{user_name}},

We received a request to reset your password. Click the link below to create a new password:

{{reset_url}}

This link expires in {{expiry_hours}} hour(s).

SECURITY NOTICE: If you didn't request this password reset, please ignore this email. Your password will remain unchanged.

---
Need help? {{help_url}}

{{company_name}} | {{company_address}}"#;

const TASK_NOTIFICATION_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Task Update</title>
</head>
<body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #f4f4f5;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; padding: 40px 20px;">
    <tr>
      <td style="background-color: #ffffff; border-radius: 8px; padding: 40px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center; padding-bottom: 30px;">
              <img src="{{logo_url}}" alt="Zerg" height="40" style="display: inline-block;">
            </td>
          </tr>
        </table>
        <h1 style="color: #18181b; font-size: 24px; font-weight: 600; margin: 0 0 16px 0; text-align: center;">
          Task {{notification_type}}: {{task_title}}
        </h1>
        {{#if task_description}}
        <p style="color: #52525b; font-size: 16px; line-height: 24px; margin: 0 0 24px 0; text-align: center;">
          {{task_description}}
        </p>
        {{/if}}
        {{#if due_date}}
        <p style="color: #71717a; font-size: 14px; margin: 0 0 24px 0; text-align: center;">
          Due: {{due_date}}
        </p>
        {{/if}}
        {{#if assigned_by}}
        <p style="color: #71717a; font-size: 14px; margin: 0 0 24px 0; text-align: center;">
          Assigned by: {{assigned_by}}
        </p>
        {{/if}}
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center;">
              <a href="{{task_url}}" style="display: inline-block; background-color: #2563eb; color: #ffffff; font-size: 16px; font-weight: 500; padding: 12px 32px; text-decoration: none; border-radius: 6px;">
                View Task
              </a>
            </td>
          </tr>
        </table>
      </td>
    </tr>
    <tr>
      <td style="padding: 24px 0; text-align: center;">
        <p style="color: #71717a; font-size: 12px; margin: 0 0 16px 0;">
          <a href="{{preferences_url}}" style="color: #2563eb; text-decoration: none;">Manage notification preferences</a>
        </p>
        <p style="color: #a1a1aa; font-size: 11px; margin: 0;">
          {{company_name}}
        </p>
      </td>
    </tr>
  </table>
</body>
</html>"#;

const TASK_NOTIFICATION_TEXT_TEMPLATE: &str = r#"Task {{notification_type}}: {{task_title}}

Hi {{user_name}},

{{#if task_description}}
{{task_description}}
{{/if}}

{{#if due_date}}
Due: {{due_date}}
{{/if}}

{{#if assigned_by}}
Assigned by: {{assigned_by}}
{{/if}}

View task: {{task_url}}

---
Manage notification preferences: {{preferences_url}}

{{company_name}}"#;

const WEEKLY_DIGEST_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Your Weekly Digest</title>
</head>
<body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: #f4f4f5;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; padding: 40px 20px;">
    <tr>
      <td style="background-color: #ffffff; border-radius: 8px; padding: 40px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center; padding-bottom: 30px;">
              <img src="{{logo_url}}" alt="Zerg" height="40" style="display: inline-block;">
            </td>
          </tr>
        </table>
        <h1 style="color: #18181b; font-size: 24px; font-weight: 600; margin: 0 0 16px 0; text-align: center;">
          Your Weekly Digest
        </h1>
        <p style="color: #52525b; font-size: 16px; line-height: 24px; margin: 0 0 24px 0; text-align: center;">
          Hi {{user_name}}, here's a summary of your week.
        </p>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-bottom: 32px;">
          <tr>
            <td style="text-align: center; padding: 16px; background-color: #f0fdf4; border-radius: 8px;">
              <p style="color: #166534; font-size: 32px; font-weight: 700; margin: 0;">{{tasks_completed}}</p>
              <p style="color: #166534; font-size: 14px; margin: 4px 0 0 0;">Tasks Completed</p>
            </td>
            <td style="width: 16px;"></td>
            <td style="text-align: center; padding: 16px; background-color: #eff6ff; border-radius: 8px;">
              <p style="color: #1d4ed8; font-size: 32px; font-weight: 700; margin: 0;">{{tasks_created}}</p>
              <p style="color: #1d4ed8; font-size: 14px; margin: 4px 0 0 0;">Tasks Created</p>
            </td>
            <td style="width: 16px;"></td>
            <td style="text-align: center; padding: 16px; background-color: #fef2f2; border-radius: 8px;">
              <p style="color: #dc2626; font-size: 32px; font-weight: 700; margin: 0;">{{tasks_overdue}}</p>
              <p style="color: #dc2626; font-size: 14px; margin: 4px 0 0 0;">Overdue</p>
            </td>
          </tr>
        </table>
        {{#if upcoming_tasks}}
        <h2 style="color: #18181b; font-size: 18px; font-weight: 600; margin: 0 0 16px 0;">
          Upcoming Tasks
        </h2>
        <table width="100%" cellspacing="0" cellpadding="0" style="margin-bottom: 32px;">
          {{#each upcoming_tasks}}
          <tr>
            <td style="padding: 12px 0; border-bottom: 1px solid #e4e4e7;">
              <a href="{{this.url}}" style="color: #18181b; font-size: 14px; text-decoration: none;">{{this.title}}</a>
              <p style="color: #71717a; font-size: 12px; margin: 4px 0 0 0;">Due: {{this.due_date}}</p>
            </td>
          </tr>
          {{/each}}
        </table>
        {{/if}}
        <table width="100%" cellspacing="0" cellpadding="0">
          <tr>
            <td style="text-align: center;">
              <a href="{{dashboard_url}}" style="display: inline-block; background-color: #18181b; color: #ffffff; font-size: 16px; font-weight: 500; padding: 12px 32px; text-decoration: none; border-radius: 6px;">
                View Dashboard
              </a>
            </td>
          </tr>
        </table>
      </td>
    </tr>
    <tr>
      <td style="padding: 24px 0; text-align: center;">
        <p style="color: #71717a; font-size: 12px; margin: 0 0 16px 0;">
          <a href="{{preferences_url}}" style="color: #2563eb; text-decoration: none;">Manage digest preferences</a>
        </p>
        <p style="color: #a1a1aa; font-size: 11px; margin: 0;">
          {{company_name}} | {{company_address}}
        </p>
      </td>
    </tr>
  </table>
</body>
</html>"#;

const WEEKLY_DIGEST_TEXT_TEMPLATE: &str = r#"Your Weekly Digest

Hi {{user_name}}, here's a summary of your week.

STATS
=====
Tasks Completed: {{tasks_completed}}
Tasks Created: {{tasks_created}}
Overdue: {{tasks_overdue}}

{{#if upcoming_tasks}}
UPCOMING TASKS
==============
{{#each upcoming_tasks}}
- {{this.title}} (Due: {{this.due_date}})
  {{this.url}}
{{/each}}
{{/if}}

View Dashboard: {{dashboard_url}}

---
Manage digest preferences: {{preferences_url}}

{{company_name}} | {{company_address}}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_render_welcome_email() {
        let engine = TemplateEngine::new().unwrap();
        let data = WelcomeEmailData {
            user_name: "Test User".to_string(),
            user_email: "test@example.com".to_string(),
            requires_verification: true,
            verification_url: Some("https://example.com/verify?token=abc123".to_string()),
            verification_expiry_hours: 24,
            dashboard_url: "https://example.com/dashboard".to_string(),
            account_type: "Free".to_string(),
            join_date: "December 13, 2025".to_string(),
            logo_url: "https://example.com/logo.png".to_string(),
            preferences_url: "https://example.com/preferences".to_string(),
            help_url: "https://example.com/help".to_string(),
            privacy_url: "https://example.com/privacy".to_string(),
            company_name: "Zerg Inc.".to_string(),
            company_address: "123 Main St".to_string(),
        };

        let result = engine.render_welcome(&data);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.html.contains("Test User"));
        assert!(rendered.text.contains("Test User"));
        assert!(rendered.subject.contains("Test User"));
    }
}
