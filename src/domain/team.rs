use anyhow::{anyhow, Result};
use std::fmt;

/// Validated email address
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    /// Create a new Email with basic validation
    pub fn new(email: &str) -> Result<Self> {
        let email = email.trim();

        // Basic email validation
        if !email.contains('@') {
            return Err(anyhow!("Invalid email format: missing '@'"));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid email format: multiple '@' symbols"));
        }

        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(anyhow!("Invalid email format: empty local or domain part"));
        }

        Ok(Email(email.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a team member
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamMember {
    pub name: String,
    pub email: Email,
    pub role: Option<String>,
}

impl TeamMember {
    /// Create a new team member
    pub fn new(name: String, email: Email) -> Self {
        TeamMember {
            name,
            email,
            role: None,
        }
    }

    /// Builder method to set the role
    pub fn with_role(mut self, role: String) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the role
    pub fn set_role(&mut self, role: String) {
        self.role = Some(role);
    }
}

/// Team with default assignee logic
#[derive(Debug, Clone)]
pub struct Team {
    members: Vec<TeamMember>,
}

impl Team {
    /// Create a new team
    pub fn new(members: Vec<TeamMember>) -> Self {
        Team { members }
    }

    /// Create an empty team
    pub fn empty() -> Self {
        Team {
            members: Vec::new(),
        }
    }

    /// Get all team members
    pub fn members(&self) -> &[TeamMember] {
        &self.members
    }

    /// Add a team member
    pub fn add_member(&mut self, member: TeamMember) {
        self.members.push(member);
    }

    /// Returns the default assignee (first team member)
    pub fn default_assignee(&self) -> Option<&TeamMember> {
        self.members.first()
    }

    /// Find a team member by name
    pub fn find_by_name(&self, name: &str) -> Option<&TeamMember> {
        self.members.iter().find(|m| m.name == name)
    }

    /// Find a team member by email
    pub fn find_by_email(&self, email: &str) -> Option<&TeamMember> {
        self.members.iter().find(|m| m.email.as_str() == email)
    }

    /// Check if the team is empty
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Get the number of team members
    pub fn len(&self) -> usize {
        self.members.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_invalid_email_no_at() {
        assert!(Email::new("testexample.com").is_err());
    }

    #[test]
    fn test_invalid_email_multiple_at() {
        assert!(Email::new("test@@example.com").is_err());
    }

    #[test]
    fn test_invalid_email_empty_parts() {
        assert!(Email::new("@example.com").is_err());
        assert!(Email::new("test@").is_err());
    }

    #[test]
    fn test_team_member() {
        let email = Email::new("john@example.com").unwrap();
        let member = TeamMember::new("John Doe".to_string(), email);
        assert_eq!(member.name, "John Doe");
        assert_eq!(member.email.as_str(), "john@example.com");
        assert!(member.role.is_none());
    }

    #[test]
    fn test_team_member_with_role() {
        let email = Email::new("john@example.com").unwrap();
        let member = TeamMember::new("John Doe".to_string(), email)
            .with_role("Developer".to_string());
        assert_eq!(member.role, Some("Developer".to_string()));
    }

    #[test]
    fn test_team_default_assignee() {
        let email1 = Email::new("john@example.com").unwrap();
        let email2 = Email::new("jane@example.com").unwrap();

        let members = vec![
            TeamMember::new("John".to_string(), email1),
            TeamMember::new("Jane".to_string(), email2),
        ];

        let team = Team::new(members);
        assert_eq!(team.default_assignee().unwrap().name, "John");
    }

    #[test]
    fn test_team_find_by_name() {
        let email = Email::new("john@example.com").unwrap();
        let member = TeamMember::new("John Doe".to_string(), email);
        let team = Team::new(vec![member]);

        assert!(team.find_by_name("John Doe").is_some());
        assert!(team.find_by_name("Jane").is_none());
    }

    #[test]
    fn test_empty_team() {
        let team = Team::empty();
        assert!(team.is_empty());
        assert_eq!(team.len(), 0);
        assert!(team.default_assignee().is_none());
    }
}
